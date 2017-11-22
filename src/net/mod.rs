//extern crate hyper;   // sometime, for a good server / client over https communication
extern crate mdns;

use self::mdns::{Record, RecordKind};
use std::net::IpAddr;


static SERVICE_NAME: &str = "_tcp.local"; // "_googlecast._tcp.local"

pub struct Net {
    my_id : String
}

impl Net {
    pub fn new(name : &String)  -> Net {
        Net { my_id : name.clone()}
    }

    pub fn lookup(&self) {
        for response in mdns::discover::all(SERVICE_NAME).unwrap() {
            match response {
                Ok(good_response) => {
                    let addr = good_response.records()
                                       .filter_map(Self::to_ip_addr)
                                       .next();

                    if let Some(addr) = addr {
                        println!("found cast device at {}", addr);
                    } else {
                        println!("cast device does not advertise address");
                    }
                }
                Err(_) => {}
            }
        }
    }

    fn to_ip_addr(record: &Record) -> Option<IpAddr> {
        match record.kind {
            RecordKind::A(addr) => Some(addr.into()),
            RecordKind::AAAA(addr) => Some(addr.into()),
            _ => None,
        }
    }
}