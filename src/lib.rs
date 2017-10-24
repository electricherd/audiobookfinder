// https://stackoverflow.com/questions/22596920/split-a-module-across-several-files
use std::sync::mpsc;

pub mod tui;
pub mod ctrl;

pub use self::tui::Tui;
//pub use self::ctrl;