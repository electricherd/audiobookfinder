use env_logger;
use syslog;

pub struct Logit {
    //
}

impl Logit {
    pub fn init(to_console: bool) {
        if !to_console {
            match syslog::unix(syslog::Facility::LOG_USER) {
                Err(e) => {
                    env_logger::init();
                    error!("impossible to connect to syslog: {:?}", e);
                }
                Ok(writer) => {
                    let r = writer.send(syslog::Severity::LOG_ALERT, "Logit init and test!");
                    if r.is_err() {
                        println!("error sending the log {}", r.err().expect("got error"));
                    }
                }
            }
        } else {
            env_logger::init();
        }
    }
}
