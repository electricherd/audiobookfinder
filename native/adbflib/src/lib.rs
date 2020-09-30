//! The adbfbinlib is a LIB and a facade to many modules that interact with
//! the audiobookfinder program.
//! In this file all crates are name before the modules which use them.
#![crate_name = "adbfbinlib"]
#![crate_type = "lib"]
// ALL
pub mod common;
pub mod ctrl;
pub mod data;
pub mod forwarder;
pub mod net;
pub mod shared;

//logger
#[macro_use]
extern crate log;

// data
#[macro_use]
extern crate serde_derive;

// config
#[macro_use]
extern crate lazy_static;

use std::{error, fmt, io};

/// A useless Error just for the Demo
#[derive(Copy, Clone, Debug)]
pub struct AdbflibError;
impl fmt::Display for AdbflibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error with adbfbinlib this page.")
    }
}
impl error::Error for AdbflibError {}
impl From<io::Error> for AdbflibError {
    fn from(_: io::Error) -> Self {
        Self
    }
}

/// the library interface for returning number of audio files found
pub async fn file_count_good(input_path: Vec<String>) -> Result<u32, AdbflibError> {
    Ok(forwarder::ffi_file_count_good(input_path))
}

/// the library interface for returning the first peer found on network
pub async fn find_new_peer() -> Result<u64, AdbflibError> {
    Ok(forwarder::ffi_new_peer().await)
}
