// https://stackoverflow.com/questions/22596920/split-a-module-across-several-files
extern crate cursive;
pub mod ctrl;

// data
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate id3;
extern crate tree_magic; // mime types
extern crate uuid;
pub mod data;

// net
extern crate mdns as io_mdns;
pub mod net;
