//! A component to use secure communication
//! using keys. It is not yet clear ...
//! basically what libp2p offers is best.
use libp2p::{
    self,
    identity::{
        ed25519::{self},
        Keypair,
        PublicKey::Ed25519,
    },
    pnet::{PnetConfig, PreSharedKey},
};
use libp2p_core::{identity, PeerId};
use std::{
    io::{Error, ErrorKind},
    path::Path,
};

pub fn get_p2p_server_id<'a>() -> PeerId {
    PeerId::from(SERVER_KEY.public())
}

lazy_static! {
    pub static ref SERVER_KEY: identity::Keypair = { identity::Keypair::generate_ed25519() };
    pub static ref PRESHARED_SECRET: PreSharedKey = {
        PreSharedKey::new([
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
            0x80, 0x80, 0x80, 0x80,
        ])
    };
}
