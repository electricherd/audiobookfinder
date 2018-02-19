use io_mdns;
use io_mdns::RecordKind;

use avahi_dns_sd;
use avahi_dns_sd::DNSService;

use std::net::IpAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::fmt::Debug;
use std::fmt::Display;
use std::thread;

use ctrl;

#[derive(Clone)]
enum Action<T: Send + Clone> {
    NewAddr(T),
    Stop,
}

static SERVICE_NAME: &str = "_http._tcpl"; // "_tcp.local"
static REGISTER_NAME: &str = "adbf";

pub struct Net {
    #[allow(dead_code)]
    my_id: String,
    addresses_found: Arc<Mutex<Vec<(u16, IpAddr)>>>,
    tui_sender: mpsc::Sender<ctrl::SystemMsg>,
    has_tui: bool,
    dns_service: DNSService
}

impl Net {
    pub fn new(
        name: &str,
        tui: bool,
        sender: mpsc::Sender<ctrl::SystemMsg>,
    ) -> Result<Net, avahi_dns_sd::DNSError> {
        //let responder = mdns::dResponse::spawn();
        let service = DNSService::register(
            Some(REGISTER_NAME),
            "_http._tcp",
            None,
            None,
            80,
            &["path=/"],
        )?;
        let net = Net {
            my_id: name.to_string(),
            addresses_found: Arc::new(Mutex::new(Vec::new())),
            tui_sender: sender,
            has_tui: tui,
            dns_service: service,
        };
        Ok(net)
    }

    pub fn lookup(&mut self) {
        // we are sending IpAddress Action
        let (send_action, receive_action) = mpsc::channel::<Action<(u16, IpAddr)>>();

        {
            // I might not need that one here
            // maybe to attach some more data
            //let borrow_arc = &self.addresses_found.clone();

            thread::spawn(move || loop {
                if let Ok(recv_mesg) = receive_action.recv() {
                    match recv_mesg {
                        Action::NewAddr(address) => {
                            thread::spawn(move || {
                                println!("received {:?} for {:?}", address.1, address.0);
                            });
                        }
                        Action::Stop => {
                            print!("stop received");
                            continue; // leave loop
                        }
                    }
                }
            });
        }
        {
            let borrow_arc = &self.addresses_found.clone();

            if let Ok(all_discoveries) = io_mdns::discover::all(SERVICE_NAME) {
                let mut count_valid = 0;
                let mut count_no_response = 0;
                let mut count_no_cast = 0;
                for (index, response) in all_discoveries.enumerate() {
                    match response {
                        Ok(good_response) => {
                            for record in good_response.records() {
                                let (out_string, addr): (
                                    Option<String>,
                                    Option<IpAddr>,
                                ) = Self::return_address(&record.kind);

                                if let Some(valid_out) = out_string {
                                    let ref mut already_found_addr =
                                        &mut borrow_arc.lock().unwrap();
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
                                                send_action
                                                    .send(Action::NewAddr(last_good.clone()))
                                                    .unwrap();
                                            }
                                        }
                                    }
                                    if self.has_tui {
                                        let host_msg = ctrl::ReceiveDialog::ShowNewHost;
                                        self.tui_sender
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
                                        self.tui_sender
                                            .send(ctrl::SystemMsg::Update(
                                                counter_msg,
                                                "".to_string(),
                                            ))
                                            .unwrap();
                                    } else {
                                        //println!("[{}] found cast device at {}", index, valid_out);
                                    }
                                } else {
                                    count_no_cast += 1;
                                    // send update
                                };
                            }
                        }
                        Err(_) => {
                            count_no_response += 1;
                        }
                    }
                }

                if !self.has_tui {
                    let output_string = format!(
                        "no response from : {no_resp:>width$}\n\
                         not castable     : {no_cast:>width$}\n",
                        no_resp = count_no_response,
                        no_cast = count_no_cast,
                        width = 3
                    );
                    println!("{}", output_string);
                }
            }
            // other thread should not wait for new messages
            send_action.send(Action::Stop).unwrap();
        }
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
