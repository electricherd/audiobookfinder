//! The StateChart section to unravel especially net related async behavior,
//! using https://github.com/fitzgen/state_machine_future to start with.
//! https://crates.io/crates/extfsm looks also promising because it supposes to use
//! standard SCXML (https://en.wikipedia.org/wiki/SCXML), has Version 0.8 yet and seems to
//!  be done on behalf of! Juniper.
//! But unfortunately yet not well documented, so choosing first mentioned one
//! (which is documented excellently in https://github.com/fitzgen/state_machine_future), though
//! version is only 0.1.6 as of now, but what does it really mean?.

mod com_client;
pub mod sc_client;
