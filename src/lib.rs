//! The adbflib is a LIB and a facade to many modules that interact with
//! the audiobookfinder program.
//! In this file all crates are name before the modules which use them.
#![crate_name = "adbflib"]
#![crate_type = "lib"]
// ALL
pub mod config;
pub mod ctrl;
pub mod logit;

//logger
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate flexi_logger;
extern crate syslog;
pub mod data;

// ctrl
// ctrl/tui
extern crate crossbeam;
extern crate cursive;
// ctrl/webui
extern crate actix;
extern crate actix_files;
extern crate actix_web;
extern crate actix_web_actors;
extern crate get_if_addrs;
extern crate hostname;
extern crate webbrowser;

// data
extern crate id3;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tree_magic; // mime types

// net
extern crate dirs;

// com_client.rs com_server.rs
extern crate futures;
extern crate futures_util;

#[macro_use]
extern crate lazy_static;

pub mod net;
