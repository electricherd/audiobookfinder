//! The client puts a state chart upfront, to make

use std::{net::IpAddr, sync::Arc, time::Duration};
use thrussh;
use uuid::Uuid;

use self::com_client::ComClient;

mod com_client;
pub mod sc_com_to;

pub struct ConnectToOther {
    connector: ComClient,
    address: IpAddr,
}

impl ConnectToOther {
    pub fn new(uuid: &Uuid, address: &IpAddr) -> ConnectToOther {
        let client = ComClient::new(uuid.clone());
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
