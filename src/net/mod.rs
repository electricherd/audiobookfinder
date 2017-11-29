//extern crate hyper;   // sometime, for a good server / client over https communication
extern crate mdns as io_mdns;

use self::io_mdns::{RecordKind};

use std::net::IpAddr;
use std::sync::mpsc;

use ctrl;


static SERVICE_NAME: &str = "_tcp.local"; // "_googlecast._tcp.local"

pub struct Net {
    my_id : String,
    my_responses : Vec<IpAddr>,
    tui_sender : mpsc::Sender<ctrl::SystemMsg>,
    has_tui : bool
}

impl Net {
    pub fn new(name : &String, tui: bool, sender: mpsc::Sender<ctrl::SystemMsg>)  -> Net {
        //let responder = mdns::dResponse::spawn(); 
        Net { my_id : name.clone(),
              my_responses : Vec::new(),
              tui_sender : sender,
              has_tui : tui
          }
    }

    pub fn lookup(&mut self) {
        if let Ok(all_discoveries) = io_mdns::discover::all(SERVICE_NAME) {
            let mut count_no_response = 0;
            let mut count_no_cast = 0;

            //let number = all_discoveries.into_iter().cloned().count();
            for (index,response) in all_discoveries.enumerate() {                
                    match response {
                        Ok(good_response) => {

                          for record in good_response.records() {
                            let (out_string, addr) : (String,Option<IpAddr>) = Self::return_address(&record.kind);

                            let text = if let Some(valid_addr) = addr {
                                self.my_responses.push(valid_addr);
                                format!(":{}:",out_string) //valid_addr)
                            } else {
                                count_no_cast += 1;
                                out_string
                            };

                            if self.has_tui {
                                let host_msg = ctrl::ReceiveDialog::ShowNewHost;
                                self.tui_sender.send(ctrl::SystemMsg::Update(host_msg,format!("found {}",text))).unwrap();
                                // send count, too
                            } else {
                                println!("[{}] found cast device at {}", index, text);                                    
                            }
                          }
                        },
                        Err(_) => { count_no_response += 1; }
                    }
            }

            if !self.has_tui {
                let output_string = format!(
                    "no response from : {no_resp:>width$}\n\
                     not castable     : {no_cast:>width$}\n"
                     ,no_resp=count_no_response
                     ,no_cast=count_no_cast
                     ,width=3);
                println!("{}",output_string);
            }
        }
    }


    fn return_address(rk : &RecordKind) -> (String,Option<IpAddr>) {
         let (out_string, addr) : (String,Option<IpAddr>) = match *rk {
            RecordKind::A(addr) => (addr.to_string(),Some(addr.into())),
            RecordKind::AAAA(addr) => (addr.to_string(),Some(addr.into())),
            RecordKind::CNAME(ref out) => (format!("{}",out.clone()),None),
            RecordKind::MX{preference,ref exchange} => (exchange.clone(),None),
            RecordKind::TXT(ref out) => (out.clone(),None),
            RecordKind::PTR(ref out) => (out.clone(),None),
            _ => { ("unknown".to_string(),None)},

        };
        (out_string,addr)
    }

}