//! The adbflib is a LIB and a facade to many modules that interact with
//! the audiobookfinder program.
//! In this file all crates are name before the modules which use them.
#![crate_name = "adbflib"]
#![crate_type = "lib"]
// ALL
pub mod common;
pub mod ctrl;
pub mod data;
pub mod net;

//logger
#[macro_use]
extern crate log;

// data
#[macro_use]
extern crate serde_derive;

// config
#[macro_use]
extern crate lazy_static;
