//! The client puts a state chart upfront, to make

use self::com_client::ComClient;
use libp2p::PeerId;
use std::{net::IpAddr, sync::Arc, time::Duration};
use thrussh;

mod com_client;
pub mod sc_com_to;

pub struct ConnectToOther {
    connector: ComClient,
    address: IpAddr,
}

impl ConnectToOther {
    pub fn new(peer_id: &PeerId, address: &IpAddr) -> ConnectToOther {
        let client = ComClient::new(peer_id.clone());
        ConnectToOther {
            connector: client,
            address: address.clone(),
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
