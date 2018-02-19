// https://stackoverflow.com/questions/22596920/split-a-module-across-several-files
//ALL
#[macro_use]
extern crate log;

// ctrl
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
extern crate mdns as io_mdns;
extern crate dns_sd as avahi_dns_sd;
pub mod net;
