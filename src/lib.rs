// https://stackoverflow.com/questions/22596920/split-a-module-across-several-files
// ALL
pub mod common;
pub mod config;

//logger
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate syslog;
extern crate flexi_logger;
pub mod logit;

// ctrl
// ctrl/tui
extern crate cursive;
pub mod ctrl;

// data
extern crate id3;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tree_magic; // mime types
extern crate uuid;
pub mod data;

// net
extern crate mdns_discover as io_mdns;
extern crate dns_sd as avahi_dns_sd;
// com_client.rs com_server.rs
extern crate futures;
extern crate thrussh;
extern crate thrussh_keys;
extern crate ring;
pub mod net;
