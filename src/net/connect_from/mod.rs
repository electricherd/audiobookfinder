///! The server section of ssh communication
pub mod com_server;

use self::com_server::ComServer;
use ring;
use std::{sync::Arc, thread, time::Duration};
use thrussh;
use thrussh_keys::key;

use super::super::config;

pub struct ConnectFromOutside {}

impl ConnectFromOutside {
    pub fn create_thread(uuid_string: String) -> Result<thread::JoinHandle<()>, ()> {
        Ok(thread::spawn(move || {
            info!("SSH ComServer starting...");

            let key_algorithm = key::ED25519;
            // possible: key::ED25519, key::RSA_SHA2_256, key::RSA_SHA2_512

            let _ = ring::rand::SystemRandom::new();
            let mut config = thrussh::server::Config::default();
            config.connection_timeout = Some(Duration::from_secs(600));
            config.auth_rejection_time = Duration::from_secs(3);
            config
                .keys
                .push(key::KeyPair::generate(key_algorithm).unwrap());
            let config = Arc::new(config);

            let replication_server = ComServer {
                name: uuid_string,
                connector: None,
            };
            let address_string = ["0.0.0.0", ":", &config::net::SSH_PORT.to_string()].concat();

            thrussh::server::run(config, &address_string, replication_server);

            warn!("SSH ComServer stopped!!");
        }))
    }
}
