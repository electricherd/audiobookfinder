//! The client puts a state chart upfront, to make

use self::com_client::ComClient;
use libp2p::PeerId;
use std::{sync::Arc, time::Duration};

mod com_client;
pub mod sc_com_to;

pub struct ConnectToOther {
    connector: ComClient,
    address: PeerId,
}

impl ConnectToOther {
    pub fn new(address: &PeerId) -> ConnectToOther {
        let copy_address = address.clone();
        let client = ComClient::new(copy_address.clone());
        ConnectToOther {
            connector: client,
            address: copy_address,
        }
    }
    pub fn run(self) {
        // todo: this needs to be done once!!!!! not for every client connection
        // move this somewhere else
        let mut config = thrussh::client::Config::default();
        config.connection_timeout = Some(Duration::from_secs(600));
        let config = Arc::new(config);

        self.connector.run(config, &self.address.clone());
    }
}
