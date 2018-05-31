//! The net module is resonsible for the network related parts,
//! the mDNS registering, mDNS search, ssh server and ssh client.
//! It also let's us startup and perform everything in yet one step.
mod data;
mod connect_from;
mod connect_to;
mod key_keeper;

use avahi_dns_sd::{self, DNSService};
use io_mdns::{self, RecordKind};
use std::{
    self, net::IpAddr, sync::{mpsc, Arc, Mutex}, thread, time::Duration,
};

use self::{connect_to::ConnectToOther, connect_from::ConnectFromOutside};
use super::{config, ctrl};

#[derive(Clone)]
enum ToThread<T: Send + Clone> {
    Data(T),
    Stop,
}

/// The Net component keeps controll about everything from net.
pub struct Net {
    my_id: String,
    addresses_found: Arc<Mutex<Vec<IpAddr>>>,
    tui_sender: mpsc::Sender<ctrl::SystemMsg>,
    has_tui: bool,
    ssh_handle: thread::JoinHandle<()>,
    // deallocate and everything will be done in drop of Net,
    // so I can:
    #[allow(dead_code)]
    dns_handle: DNSService,
}

impl Net {
    pub fn new(
        name: &str,
        tui: bool,
        sender: mpsc::Sender<ctrl::SystemMsg>,
    ) -> Result<Net, avahi_dns_sd::DNSError> {
        //let responder = mdns::dResponse::spawn();

        // the drop of self.dns_handle will unregister
        // so I need to keep it like here :-(
        let dns_service = DNSService::register(
            Some(config::net::MDNS_REGISTER_NAME),
            config::net::MDNS_SERVICE_NAME,
            None,
            None,
            config::net::MDNS_PORT,
            &["path=/"],
        )?;
        let net = Net {
            my_id: name.to_string(),
            addresses_found: Arc::new(Mutex::new(Vec::new())),
            tui_sender: sender,
            has_tui: tui,
            ssh_handle: thread::spawn(|| {}), // empty join handle
            dns_handle: dns_service,
        };
        Ok(net)
    }

    pub fn start_com_server(&mut self) -> Result<(), ()> {
        // delegate this somewhere else
        if key_keeper::is_good() {
            if let Ok(good_thread) = ConnectFromOutside::create_thread(self.my_id.clone()) {
                self.ssh_handle = good_thread;
                Ok(())
            } else {
                Err(())
            }
        } else {
            // ToDo: error
            Err(())
        }
    }

    /// Lookup yet is the start of the networking.
    /// It looks for possible mDNS clients and spawns
    // threads to connect to them.
    // It uses timeouts, checkups.
    pub fn lookup(&mut self) {
        // we are sending IpAddress ToThread

        let (sender_ssh_client, receiver_client) = mpsc::channel::<ToThread<IpAddr>>();
        let sender_ssh_client_stop = sender_ssh_client.clone();
        // I might not need that one here
        // maybe to attach some more data
        //let borrow_arc = &self.addresses_found.clone();
        let has_tui = self.has_tui;
        let uuid_name = self.my_id.clone();

        // to controller messages (mostly tui now)
        let ctrl_sender = self.tui_sender.clone();
        let ctrl_sender2 = self.tui_sender.clone();

        // collection of addresses
        let borrow_arc = self.addresses_found.clone();

        let _get_ip_thread = thread::spawn(move || {
            //
            //
            Self::spawn_connect_new_clients_threads(
                receiver_client,
                borrow_arc,
                ctrl_sender,
                uuid_name,
                has_tui,
            );
        });

        // use a timeout to stop search after a time
        // the nice one should work stop gracefully,
        let (timeout_nice_sender, timeout_nice_receiver) = mpsc::channel();
        let (timeout_nice_renewer, timeout_nice_recover) = mpsc::channel();
        let _timeout_graceful_thread = thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(config::net::MDNS_TIMEOUT_SEC as u64));
            // if has been recovered until here ... continue loop
            match timeout_nice_recover.try_recv() {
                Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                    debug!("graceful time out sent....");
                    timeout_nice_sender.send(()).unwrap();
                    break; // leave loop and _timeout_graceful_thread
                }
                Err(mpsc::TryRecvError::Empty) => {
                    debug!("graceful time out suspended");
                }
            }
        });

        // statistics
        let mut count_valid = 0;
        let mut count_no_cast = 0;
        let mut count_no_response = 0;

        // prepare everything for mdns thread
        let (mdns_send_ip, mdns_receive_ip) = mpsc::channel::<ToThread<io_mdns::Record>>();

        let mdns_send_stop = mdns_send_ip.clone();

        // take input from mdns_discoper_thread
        // (new ssh client discovered, or renew timeout).
        // but also sends if time out a timeout back to
        // mdns_discover.
        // If client good send sender_ssh_client to
        // _get_ip_thread
        let _take_mdns_input_thread = thread::spawn(move || {
            Self::take_mdns_input(
                mdns_receive_ip,
                sender_ssh_client,
                timeout_nice_renewer,
                ctrl_sender2,
                has_tui,
                // change these
                &mut count_no_cast,
                &mut count_valid,
            );
        });

        let mdns_discover_thread = thread::spawn(move || {
            count_no_response = Self::mdns_discover(mdns_send_ip, timeout_nice_receiver);
            if !has_tui {
                let output_string = format!(
                    "no response from : {no_resp:>width$}\n\
                     not castable     : {no_cast:>width$}\n",
                    no_resp = count_no_response,
                    no_cast = count_no_cast,
                    width = 3
                );
                info!("{}", output_string);
            }
        });

        // yes, finally we wait for this thread
        let _ = mdns_discover_thread.join();

        // other thread should not wait for new messages after all
        sender_ssh_client_stop.send(ToThread::Stop).unwrap();
        mdns_send_stop.send(ToThread::Stop).unwrap();
    }

    fn return_address(rk: &RecordKind) -> (Option<String>, Option<IpAddr>) {
        let (out_string, addr): (Option<String>, Option<IpAddr>) = match *rk {
            RecordKind::A(addr) => (Some(addr.to_string()), Some(addr.into())),
            RecordKind::AAAA(addr) => (Some(addr.to_string()), Some(addr.into())),
            RecordKind::CNAME(ref out) => (Some(format!("{}", out.clone())), None),
            RecordKind::MX { ref exchange, .. } => (Some(exchange.clone()), None),
            RecordKind::TXT(ref out) => (Some(out.clone()), None),
            RecordKind::PTR(ref out) => (Some(out.clone()), None),
            _ => (None, None),
        };
        (out_string, addr)
    }

    fn mdns_discover(
        mdns_send_ip: std::sync::mpsc::Sender<ToThread<io_mdns::Record>>,
        timeout_nice_receiver: std::sync::mpsc::Receiver<()>,
    ) -> usize {
        // mdns_send_ip  std::sync::mpsc::Sender<net::ToThread<(usize, io_mdns::Record)>>
        //timeout_nice_receiver  std::sync::mpsc::Receiver<()>
        let mut count_no_response = 0;

        // must be combined
        let full_name = [
            config::net::MDNS_REGISTER_NAME,
            config::net::MDNS_SERVICE_NAME,
        ].join(".");
        info!("Searching for {:?}!", full_name);

        if let Ok(all_discoveries) = io_mdns::discover::all(full_name) {
            info!("MDNS search: starting");
            // this is the long search loop
            // looking rather inefficient, because cpu goes crazy
            // don't think this is my fault
            for (_index, response) in all_discoveries.enumerate() {
                // if message came or sender gone continue and leave loop
                // don't just kill all of this
                match timeout_nice_receiver.try_recv() {
                    Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                        warn!(
                            "MDNS search: timeout received (should have been {:?} sec)",
                            config::net::MDNS_TIMEOUT_SEC
                        );
                        break; // leave loop
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        // do nothing
                    }
                }

                // just look response
                match response {
                    Ok(good_response) => for record in good_response.records() {
                        mdns_send_ip.send(ToThread::Data(record.clone())).unwrap();
                    },
                    Err(_) => {
                        count_no_response += 1;
                    }
                }
            } //end for loop
        } // for loop
        count_no_response
    }

    fn take_mdns_input(
        mdns_receive_ip: std::sync::mpsc::Receiver<ToThread<io_mdns::Record>>,
        sender_ssh_client: std::sync::mpsc::Sender<ToThread<IpAddr>>,
        timeout_nice_renewer: std::sync::mpsc::Sender<()>,
        ctrl_sender: std::sync::mpsc::Sender<ctrl::SystemMsg>,
        has_tui: bool,
        count_no_cast: &mut usize,
        count_valid: &mut usize,
    ) {
        loop {
            match mdns_receive_ip.recv() {
                Ok(good) => {
                    match good {
                        ToThread::Data(recv_mesg) => {
                            let (_, addr): (
                                Option<String>,
                                Option<IpAddr>,
                            ) = Self::return_address(&recv_mesg.kind);
                            if let Some(valid_addr) = addr {
                                *count_valid += 1;
                                sender_ssh_client
                                    .send(ToThread::Data(valid_addr.clone()))
                                    .unwrap();
                                // renew time out
                                timeout_nice_renewer.send(()).unwrap();
                            }
                        }
                        ToThread::Stop => {
                            break; //break loop
                        }
                    }
                }
                Err(_) => {
                    *count_no_cast += 1;
                    break; //break loop
                }
            }
        } // loop ended
          // finally stop animation
        if has_tui {
            info!("Stop animation!");
            ctrl_sender
                .send(ctrl::SystemMsg::StartAnimation(
                    ctrl::Alive::HostSearch,
                    ctrl::Status::OFF,
                ))
                .unwrap();
        }
    }

    fn spawn_connect_new_clients_threads(
        receiver_client: std::sync::mpsc::Receiver<ToThread<IpAddr>>,
        borrow_arc: std::sync::Arc<std::sync::Mutex<std::vec::Vec<IpAddr>>>,
        ctrl_sender: std::sync::mpsc::Sender<ctrl::SystemMsg>,
        uuid_name: String,
        has_tui: bool,
    ) {
        loop {
            // wait for message
            if let Ok(recv_mesg) = receiver_client.recv() {
                match recv_mesg {
                    ToThread::Data(address) => {
                        //
                        // input address
                        //
                        let try_lock = &mut borrow_arc.lock();
                        if let &mut Ok(ref mut ip_addresses) = try_lock {
                            // search if ip address is already collected
                            if ip_addresses.iter().all(|ip| *ip != address) {
                                // put into collection to not find again
                                ip_addresses.push(address);

                                let count = ip_addresses.len();
                                if has_tui {
                                    ctrl_sender
                                        .send(ctrl::SystemMsg::Update(
                                            ctrl::ReceiveDialog::ShowNewHost,
                                            address.to_string(),
                                        ))
                                        .unwrap();
                                    ctrl_sender
                                        .send(ctrl::SystemMsg::Update(
                                            ctrl::ReceiveDialog::ShowStats {
                                                show: ctrl::NetStats {
                                                    line: count,
                                                    max: 0, //index,
                                                },
                                            },
                                            "".to_string(),
                                        ))
                                        .unwrap();
                                }
                                // 2nd embedded and in loop-thread so copy copied again
                                let uuid_name = uuid_name.clone();

                                // create ssh client in new thread
                                let _connect_ip_client_thread = thread::spawn(move || {
                                    let connector = ConnectToOther::new(&uuid_name, &address);
                                    connector.run();
                                });
                            }
                        } else {
                            error!("Could not lock ip address list!");
                        }
                    }
                    ToThread::Stop => {
                        break; // leave loop and _get_ip_thread
                    }
                }
            } else {
                break; // leave loop and _get_ip_thread
            }
        }
    }
}

impl Drop for Net {
    fn drop(&mut self) {
        std::mem::forget(&self.ssh_handle);
        if !self.has_tui {
            println!("Dropping/destroying net");
        }
    }
}
