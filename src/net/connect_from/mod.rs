///! The server section of ssh communication
pub mod com_server;

use self::com_server::ComServer;
use ring;
use std::{sync::Arc, thread, time::Duration};
use thrussh;
use thrussh_keys::key;
use uuid::Uuid;

use super::super::config;

pub struct ConnectFromOutside {}

impl ConnectFromOutside {
    pub fn create_thread(uuid: Uuid) -> Result<thread::JoinHandle<()>, ()> {
        Ok(thread::spawn(move || {
            info!("SSH ComServer starting...");

            let _ = ring::rand::SystemRandom::new();
            let mut config = thrussh::server::Config::default();
            config.connection_timeout = Some(Duration::from_secs(600));
            config.auth_rejection_time = Duration::from_secs(3);
            config.keys.push(key::KeyPair::generate_ed25519().unwrap());
            let config = Arc::new(config);

            let replication_server = ComServer { id: uuid };
            let address_string = ["0.0.0.0", ":", &config::net::PORT_SSH.to_string()].concat();

            thrussh::server::run(config, &address_string, replication_server);

            warn!("SSH ComServer stopped!!");
        }))
    }
}
