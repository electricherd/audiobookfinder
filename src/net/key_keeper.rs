//! A component to use secure communication
//! using keys. It is not yet clear ...
//! basically what libp2p offers is best.
use libp2p::{self, pnet::PreSharedKey};
use libp2p_core::{identity, PeerId};

pub fn get_p2p_server_id<'a>() -> PeerId {
    PeerId::from(SERVER_KEY.public())
}

lazy_static! {
    pub static ref SERVER_KEY: identity::Keypair = identity::Keypair::generate_ed25519();
    pub static ref PRESHARED_SECRET: PreSharedKey = {
        PreSharedKey::new([
            0x23, 0x89, 0xb4, 0x82, 0x42, 0x89, 0x7e, 0x8f, 0x54, 0x85, 0xd1, 0x3e, 0xd1, 0x2e,
            0xaf, 0x33, 0xc1, 0x44, 0x86, 0x89, 0xde, 0x8c, 0x21, 0xc5, 0x82, 0x8d, 0xe7, 0x70,
            0x34, 0x21, 0x74, 0xc9,
        ])
    };
}
