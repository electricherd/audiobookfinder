///! The server section of ssh communication
pub mod com_server;

use libp2p::PeerId;
use std::io::Result;

pub struct ConnectFromOutside {}

impl ConnectFromOutside {
    #[allow(dead_code)]
    pub async fn create_thread(_peer_id: PeerId) -> Result<()> {
        info!("SSH ComServer starting...");

        // let _ = ring::rand::SystemRandom::new();
        // let mut config = thrussh::server::Config::default();
        // config.connection_timeout = Some(Duration::from_secs(600));
        // config.auth_rejection_time = Duration::from_secs(3);
        // config.keys.push(key::KeyPair::generate_ed25519().unwrap());
        // let config = Arc::new(config);
        //
        // let replication_server = ComServer { peer_id: peer_id };
        // let address_string = ["0.0.0.0", ":", &config::net::PORT_SSH.to_string()].concat();
        //
        // let mut tokio_rt = runtime::Runtime::new().unwrap();
        // tokio_rt.block_on(async move {
        //     thrussh::server::run(config, &address_string, replication_server)
        // });

        warn!("SSH ComServer stopped!!");
        Ok(())
    }
}
