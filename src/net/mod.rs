use std;
use std::fmt::Debug;
use std::fmt::Display;
use std::net::IpAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use io_mdns;
use io_mdns::RecordKind;

use avahi_dns_sd;
use avahi_dns_sd::DNSService;

use thrussh;
use thrussh_keys::key;

use ring;

use config;
use ctrl;

pub mod com_server;
pub mod com_client;

type IpType = (u16, IpAddr);

#[derive(Clone)]
enum ToThread<T: Send + Clone> {
    Data(T),
    Stop,
}

pub struct Net {
    my_id: String,
    addresses_found: Arc<Mutex<Vec<IpType>>>,
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
        // well is this an attached thread ????

        let uuid_name = self.my_id.clone();
        self.ssh_handle = thread::spawn(move || {
            info!("SSH ComServer starting...");

            let key_algorithm = key::ED25519;
            // possible: key::ED25519, key::RSA_SHA2_256, key::RSA_SHA2_512

            let _ = ring::rand::SystemRandom::new();
            let mut config = thrussh::server::Config::default();
            config.connection_timeout = Some(Duration::from_secs(600));
            config.auth_rejection_time = Duration::from_secs(3);
            config
                .keys
                .push(key::KeyPair::generate(key_algorithm).unwrap());
            let config = Arc::new(config);

            let replication_server = com_server::ComServer {
                name: uuid_name,
                connector: None,
            };
            thrussh::server::run(config, config::net::SSH_CLIENT_AND_PORT, replication_server);
            warn!("SSH ComServer stopped!!");
        });
        Ok(())
    }

    pub fn lookup(&mut self) {
        // we are sending IpAddress ToThread

        let (sender_ssh_client, receiver_client) = mpsc::channel::<ToThread<IpType>>();
        let sender_ssh_client_stop = sender_ssh_client.clone();
        // I might not need that one here
        // maybe to attach some more data
        //let borrow_arc = &self.addresses_found.clone();
        let has_tui = self.has_tui;
        let uuid_name = self.my_id.clone();

        let _get_ip_thread = thread::spawn(move || {
            //
            //
            Self::spawn_connect_new_clients_threads(receiver_client, uuid_name);
        });

        // use a raw timeout to stop search after a time
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

        let mut count_valid = 0;
        let mut count_no_cast = 0;
        let mut count_no_response = 0;

        // prepare everything for mdns thread
        let ctrl_sender = self.tui_sender.clone();
        let (mdns_send_ip, mdns_receive_ip) = mpsc::channel::<ToThread<(usize, io_mdns::Record)>>();

        let mdns_send_stop = mdns_send_ip.clone();
        let borrow_arc = self.addresses_found.clone();

        // keep the mdns stuff in a separate thread
        let _take_mdns_input_thread = thread::spawn(move || {
            Self::take_mdns_input(
                mdns_receive_ip,
                borrow_arc,
                sender_ssh_client,
                timeout_nice_renewer,
                ctrl_sender,
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

    // A simple function from seen above (but implementing this actually took a while)
    // But I tried to implement some Generic parts
    // and also trying iterator reverse, mut borrowing in some funny ways (with internal
    // mutibility)
    fn could_add_addr<T1, T2>(index: T1, input: T2, out: &mut Vec<(T1, T2)>) -> bool
    where
        T1: PartialEq + Clone,
        T2: Clone + PartialOrd + Display + Debug, // Display Debug for println output
    {
        if out.iter().find(|&e| e.0 == index).is_none() {
            out.push((index, input));
            true
        } else {
            // only store one for each index
            // just a training for generics in Rust, but since I only use 1 value per index
            // I wanted to search all from end up
            // for the following I needed  + Clone for cloned()
            //                   and        PartialEq for e.0 == index
            //let same_index : Vec<(T1,T2)> = out.iter().rev().cloned().filter(|e| e.0 == index).collect();
            //assert!(same_index.len() == 1);
            let mut same_indeces: Vec<&mut (T1, T2)> =
                out.iter_mut().rev().filter(|e| e.0 == index).collect();

            // since we checked before if find finds something, also this "find" or collect
            // should find exactly 1 (since we replace every single one)
            let ref mut comparer = &mut *same_indeces[0];
            if comparer.1 > input {
                debug!("{} replaced by {}", input, comparer.1);
                comparer.1 = input;
            }
            false
        }
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
        mdns_send_ip: std::sync::mpsc::Sender<ToThread<(usize, io_mdns::Record)>>,
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
            for (index, response) in all_discoveries.enumerate() {
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
                        mdns_send_ip
                            .send(ToThread::Data((index, record.clone())))
                            .unwrap();
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
        mdns_receive_ip: std::sync::mpsc::Receiver<ToThread<(usize, io_mdns::Record)>>,
        borrow_arc: std::sync::Arc<std::sync::Mutex<std::vec::Vec<IpType>>>,
        sender_ssh_client: std::sync::mpsc::Sender<ToThread<IpType>>,
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
                        ToThread::Data((index, recv_mesg)) => {
                            let (out_string, addr): (
                                Option<String>,
                                Option<IpAddr>,
                            ) = Self::return_address(&recv_mesg.kind);
                            if let Some(valid_out) = out_string {
                                let ref mut already_found_addr = &mut borrow_arc.lock().unwrap();
                                if let Some(valid_addr) = addr {
                                    if Self::could_add_addr(
                                        index as u16,
                                        valid_addr,
                                        already_found_addr,
                                    ) {
                                        *count_valid += 1;

                                        // get the last good index, this is the final
                                        // ipv6 address to send request
                                        if let Some(last_good) = already_found_addr.last() {
                                            sender_ssh_client
                                                .send(ToThread::Data(last_good.clone()))
                                                .unwrap();
                                            // renew time out
                                            timeout_nice_renewer.send(()).unwrap();
                                            // send name to tui
                                            if has_tui {
                                                ctrl_sender
                                                    .send(ctrl::SystemMsg::Update(
                                                        ctrl::ReceiveDialog::ShowNewHost,
                                                        valid_out,
                                                    ))
                                                    .unwrap();
                                                ctrl_sender
                                                    .send(ctrl::SystemMsg::Update(
                                                        ctrl::ReceiveDialog::ShowStats {
                                                            show: ctrl::NetStats {
                                                                line: *count_valid,
                                                                max: index,
                                                            },
                                                        },
                                                        "".to_string(),
                                                    ))
                                                    .unwrap();
                                            } else {
                                                info!("accepted address: {}", valid_out);
                                            }
                                        }
                                    }
                                }
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
        receiver_client: std::sync::mpsc::Receiver<ToThread<IpType>>,
        uuid_name: String,
    ) {
        loop {
            if let Ok(recv_mesg) = receiver_client.recv() {
                match recv_mesg {
                    ToThread::Data(address) => {
                        // 2nd embedded and in loop-thread so copy copied again
                        let uuid_name = uuid_name.clone();

                        let _connect_ip_client_thread = thread::spawn(move || {
                            debug!("pretending as if trying {:?}!", address);

                            // wait a bit until ssh server is up. This will be moved somewhere else anyway
                            thread::sleep(Duration::from_millis(500));
                            // example client, will be done correctly if mDNS finds other instances
                            let mut config = thrussh::client::Config::default();
                            config.connection_timeout = Some(Duration::from_secs(600));
                            let config = Arc::new(config);
                            let client = com_client::ComClient::new(uuid_name);
                            if client.run(config, config::net::SSH_HOST_AND_PORT).is_err() {
                                error!("SSH Client example not working!!!");
                            }
                        });
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
