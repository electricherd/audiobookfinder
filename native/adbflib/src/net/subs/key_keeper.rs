//! A component to use secure communication
//! using keys. It is not yet clear ...
//! basically what libp2p offers is best.
use libp2p::{
    self,
    core::{identity, PeerId},
    pnet::PreSharedKey,
};

/// Gets a newly generated server ID
pub fn get_p2p_server_id<'a>() -> PeerId {
    PeerId::from(SERVER_KEY.public())
}

lazy_static! {
    /// Generates a new ed25519 key-pair
    pub static ref SERVER_KEY: identity::Keypair = identity::Keypair::generate_ed25519();
    /// This is the to be hidden/read preshare key for the net communication process
    pub static ref PRESHARED_SECRET: PreSharedKey = {
        let binary_from_file :&'static [u8;32] = include_bytes!("secret.bin");
        PreSharedKey::new(*binary_from_file)
    };
}
