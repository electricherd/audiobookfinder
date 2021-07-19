//! This module lets us be more flexible with logging.
//! It provides different types of loggers for different purposes, if run in tui or
//! somewhere (maybe a syslog log).
use env_logger;
use flexi_logger::{self, FileSpec};
use syslog::{self, Facility, Formatter3164};

pub enum Log {
    Console,
    File,
    System,
}

/// The logger for all log-types: warn, info, trace, error, debug
pub struct Logit {}

/// The very practical logger. It can use 3 types of logging yet.
///  - system logger
///  - console logging
///  - file logging (used)
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
                // now only from console, good is now:
                //       RUST_LOG=audiobookfinder=trace,adbflib=trace
                //       or
                //         RUST_LOG=adbfbinlib=trace
                //         RUST_LOG=adbfbinlib::net=trace
                //         RUST_LOG=audiobookfinder=trace,adbfbinlib=trace
                //
                //       and
                //         ADBF_LOG=file (or console,system)
                // see:
                // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
                flexi_logger::Logger::try_with_env_or_str("info")
                    .unwrap()
                    .log_to_file(
                        FileSpec::default()
                            .directory(".")
                            .suppress_timestamp()
                            .suffix("log"),
                    )
                    .format(flexi_logger::with_thread) // colored_with_thread
                    .start()
                    .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
            }
        }
    }
}

/// Read env level given e.g. by command-line, environment variable
pub fn read_env_level(level: &str) -> Log {
    match level {
        "console" => Log::Console,
        "file" => Log::File,
        "system" => Log::System,
        _ => Log::System,
    }
}
