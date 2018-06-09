//! The adbflib is a facade to many modules that interact with
//! the audiobookfinder program.
//! In this file all crates are name before the modules which use them.
#![crate_name = "adbflib"]
#![crate_type = "lib"]
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
#[macro_use]
extern crate state_machine_future;
pub mod ctrl;

// data
extern crate taglib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tree_magic; // mime types
extern crate uuid;
pub mod data;

// net
extern crate bincode;
extern crate mdns_discover as io_mdns;
extern crate dns_sd as avahi_dns_sd;
// com_client.rs com_server.rs
extern crate futures;
extern crate thrussh;
extern crate thrussh_keys;
extern crate ring;
extern crate tokio_io;
#[macro_use]
extern crate lazy_static;


pub mod net;
