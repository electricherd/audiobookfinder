//! This module lets us be more flexible wiht logging.
//! It provides different types of loggers for different purposes, if run in tui or
//! somewhere (maybe a syslog log).
use env_logger;
use flexi_logger;
use syslog::{self, Facility, Formatter3164};

pub enum Log {
    Console,
    File,
    System,
}

pub struct Logit {
    //
}

/// Uses 3 types of logging yet.
impl Logit {
    pub fn init(which: Log) {
        match which {
            Log::System => {
                let formatter = Formatter3164 {
                    facility: Facility::LOG_USER,
                    hostname: None,
                    process: env!("CARGO_PKG_NAME").into(),
                    pid: 42,
                };

                match syslog::unix(formatter) {
                    Err(e) => {
                        env_logger::init();
                        error!("impossible to connect to syslog: {:?}", e);
                    }
                    Ok(mut writer) => {
                        writer
                            .err("Logit init and test!")
                            .expect("could not write error message");
                    }
                }
            }
            Log::Console => {
                env_logger::init();
            }
            Log::File => {
                // todo: fix this for binary and see for default!
                //       from console good is now:
                //       RUST_LOG=audiobookfinder=trace,adbflib=trace
                // see:
                // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
                flexi_logger::Logger::with_env_or_str("adbflib=debug, adbflib=warn")
                    .log_to_file()
                    .directory(".")
                    .format(flexi_logger::with_thread) // colored_with_thread
                    .suppress_timestamp()
                    .suffix("log")
                    .start()
                    .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
            }
        }
    }
}
