use env_logger;
use flexi_logger;
use syslog;

pub enum Log {
    Console,
    File,
    System,
}

pub struct Logit {
    //
}

impl Logit {
    pub fn init(which: Log) {
        match which {
            Log::System => match syslog::unix(syslog::Facility::LOG_USER) {
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
            },
            Log::Console => {
                env_logger::init();
            }
            Log::File => {
                flexi_logger::Logger::with_env_or_str("adbflib=debug, adbflib=warn")
                    .log_to_file()
                    .directory(".")
                    .format(flexi_logger::opt_format)
                    .suppress_timestamp()
                    .suffix("log")
                    .start()
                    .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
            }
        }
    }
}
