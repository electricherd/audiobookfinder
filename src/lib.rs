// https://stackoverflow.com/questions/22596920/split-a-module-across-several-files
use std::sync::mpsc;


pub mod ctrl;
pub mod data;

//pub use self::ctrl;