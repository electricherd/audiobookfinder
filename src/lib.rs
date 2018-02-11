// https://stackoverflow.com/questions/22596920/split-a-module-across-several-files
// ctrl
extern crate cursive;
// data
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate id3;
extern crate tree_magic; // mime types
extern crate uuid;
// net
extern crate mdns as io_mdns;

pub mod ctrl;
pub mod data;
pub mod net;
