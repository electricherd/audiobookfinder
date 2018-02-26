use io_mdns;
use io_mdns::RecordKind;

use avahi_dns_sd;
use avahi_dns_sd::DNSService;

//use ring;

use thrussh;
use thrussh_keys::key;

use std;
use std::net::IpAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::fmt::Debug;
use std::fmt::Display;
use std::thread;
use std::time::Duration;

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
    #[allow(dead_code)]
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
        let has_tui = self.has_tui;
        // well is this an attached thread ????
        self.ssh_handle = thread::spawn(move || {
            if !has_tui {
                println!("SSH ComServer starting...");
            }
            //let rand = ring::rand::SystemRandom::new();
            let mut config = thrussh::server::Config::default();

            let key_algorithm = key::ED25519;
            // possible: key::ED25519, key::RSA_SHA2_256, key::RSA_SHA2_512

            config.connection_timeout = Some(Duration::from_secs(600));
            config.auth_rejection_time = Duration::from_secs(3);
            config
                .keys
                .push(key::KeyPair::generate(key_algorithm).unwrap());
            let config = Arc::new(config);
            let sh = com_server::ComServer {};
            thrussh::server::run(config, config::net::SSH_CLIENT_AND_PORT, sh);
            if !has_tui {
                println!("SSH ComServer stopped!!");
            }
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
        thread::spawn(move || loop {
            if let Ok(recv_mesg) = receiver_client.recv() {
                match recv_mesg {
                    ToThread::Data(address) => {
                        thread::spawn(move || {
                            if !has_tui {
                                println!("pretending as if trying {:?}!", address);
                            }
                            // wait a bit until ssh server is up. This will be moved somewhere else anyway
                            thread::sleep(Duration::from_millis(500));
                            // example client, will be done correctly if mDNS finds other instances
                            let mut config = thrussh::client::Config::default();
                            config.connection_timeout = Some(Duration::from_secs(600));
                            let config = Arc::new(config);
                            let client = com_client::ComClient {};
                            if client.run(config, config::net::SSH_HOST_AND_PORT).is_err() {
                                if !has_tui {
                                    println!("SSH Client example not working!!!");
                                }
                            }
                        });
                    }
                    ToThread::Stop => {
                        break; // leave loop and thread
                    }
                }
            } else {
                break; // leave loop and thread
            }
        });

        let mut count_valid = 0;
        let mut count_no_response = 0;
        let mut count_no_cast = 0;

        // use a raw timeout to stop search after a time
        // the nice one should work stop gracefully,
        // the bad one is necessary due to never ending
        // io_mdns::discover::all f*cking up
        //let (timeout_bad_sender, timeout_bad_receiver) = mpsc::channel();
        let (timeout_nice_sender, timeout_nice_receiver) = mpsc::channel();
        let (timeout_nice_renewer, timeout_nice_recover) = mpsc::channel();
        let _timeout_graceful = thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(config::net::MDNS_TIMEOUT_SEC as u64));
            // if has been recovered until here ... continue loop
            match timeout_nice_recover.try_recv() {
                Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                    if !has_tui {
                        println!("graceful time out sent....");
                    }
                    timeout_nice_sender.send(()).unwrap();
                    break; // leave loop
                }
                Err(mpsc::TryRecvError::Empty) => {
                    if !has_tui {
                        println!("graceful time out suspended");
                    }
                }
            }
        });

        // prepare everything for mdns thread
        let ctrl_sender = self.tui_sender.clone();
        let (mdns_send_ip, mdns_receive_ip) = mpsc::channel::<ToThread<(usize, io_mdns::Record)>>();

        let mdns_send_stop = mdns_send_ip.clone();
        let borrow_arc = self.addresses_found.clone();

        // keep the mdns stuff in a separate thread
        let _take_mdns_input = thread::spawn(move || loop {
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
                                        count_valid += 1;

                                        // get the last good index, this is the final
                                        // ipv6 address to send request
                                        if let Some(last_good) = already_found_addr.last() {
                                            sender_ssh_client
                                                .send(ToThread::Data(last_good.clone()))
                                                .unwrap();
                                            // renew time out
                                            timeout_nice_renewer.send(()).unwrap();
                                        }
                                    }
                                }
                                if has_tui {
                                    let host_msg = ctrl::ReceiveDialog::ShowNewHost;
                                    ctrl_sender
                                        .send(ctrl::SystemMsg::Update(
                                            host_msg,
                                            format!("found {}", valid_out),
                                        ))
                                        .unwrap();
                                    let counter_msg = ctrl::ReceiveDialog::ShowStats {
                                        show: ctrl::NetStats {
                                            line: count_valid,
                                            max: index,
                                        },
                                    };
                                    ctrl_sender
                                        .send(ctrl::SystemMsg::Update(counter_msg, "".to_string()))
                                        .unwrap();
                                } else {
                                    //println!("[{}] found cast device at {}", index, valid_out);
                                }
                            }
                        }
                        ToThread::Stop => {
                            break; //break loop
                        }
                    }
                }
                Err(_) => {
                    count_no_cast += 1;
                    break; //break loop
                }
            }
        });

        let mdns_thread = thread::spawn(move || {
            if let Ok(all_discoveries) = io_mdns::discover::all(config::net::MDNS_SERVICE_NAME) {
                if !has_tui {
                    println!("MDNS search: starting");
                }
                // this is the long search loop
                // looking rather inefficient, because cpu goes crazy
                // don't think this is my fault
                for (index, response) in all_discoveries.enumerate() {
                    // if message came or sender gone continue and leave loop
                    // don't just kill all of this
                    match timeout_nice_receiver.try_recv() {
                        Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                            if !has_tui {
                                println!(
                                    "MDNS search: timeout received (should have been {:?} sec)",
                                    config::net::MDNS_TIMEOUT_SEC
                                );
                            }
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

                if !has_tui {
                    let output_string = format!(
                        "no response from : {no_resp:>width$}\n\
                         not castable     : {no_cast:>width$}\n",
                        no_resp = count_no_response,
                        no_cast = count_no_cast,
                        width = 3
                    );
                    println!("{}", output_string);
                }
            } // for loop
        });

        // yes, finally we wait for this thread
        let _ = mdns_thread.join();

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
                println!("{} replaced by {}", input, comparer.1);
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
}

impl Drop for Net {
    fn drop(&mut self) {
        std::mem::forget(&self.ssh_handle);
        if !self.has_tui {
            println!("Dropping/destroying net");
        }
    }
}
