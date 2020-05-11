//! A component to use key

use super::super::config;
use dirs;
use libp2p::identity::PublicKey::Ed25519;
use libp2p::{
    self,
    identity::ed25519::{self, PublicKey},
    PeerId,
};
use std::{self, sync::Arc};
use thrussh;
use thrussh_keys;

pub fn get_p2p_server_public_key<'a>() -> PublicKey {
    (&*SERVER_KEY).public()
}

pub fn get_p2p_server_id<'a>() -> PeerId {
    PeerId::from_public_key(Ed25519(get_p2p_server_public_key()))
}

lazy_static! {
    pub static ref SERVER_KEY_SSH : Arc<thrussh_keys::key::KeyPair> = {
        let priv_key_path = config::net::SSH_CLIENT_SEC_KEY_PATH;
        let static_passwd = config::net::SSH_CLIENT_SEC_KEY_PASSWD;

        // home is type changed, so always new ...
        let key_from_file = dirs::home_dir().and_then( |home| {
            home.to_str().and_then( |m_home| {
                let filename = [&m_home, priv_key_path].concat();
                match std::fs::read(&filename) {
                    Ok(buffer) => {
                        Some(thrussh::CryptoVec::from_slice(&buffer))
                    },
                    Err(_) => {
                        error!("Not found or password wrong: {:?}", &filename);
                        None
                    }
                }
            }).or_else( || {
                error!("Path to key file has illegal symbols!");
                None
            }
            )
        }).or_else(|| {
            error!("Home dir not set!");
            None
        });

        let decoded = if let Some(key_slice) = key_from_file {
            key_slice
                .get(..)
                .and_then(|sliced| match std::str::from_utf8(sliced) {
                    Ok(valid_utf8) => match thrussh_keys::decode_secret_key(valid_utf8, Some(static_passwd.as_bytes())) {
                        Ok(good) => Some(good),
                        Err(_) => {
                            error!("Is not encrypted with static password!");
                            None
                        }
                    },
                    Err(_) => {
                        error!("Some encoding issues with ssh key!");
                        None
                    }
                })
        } else {
            error!("file could not be decoded!");
            None
        };

        let the_key = if let Some(good) = decoded {
            good
        } else {
            thrussh_keys::key::KeyPair::generate_ed25519().unwrap()
        };
        Arc::new(the_key)
    };


    pub static ref SERVER_KEY : ed25519::Keypair = {
        let priv_key_path = config::net::SSH_CLIENT_SEC_KEY_PATH;

        // home is type changed, so always new ...
        let key_reading_from_file_system = dirs::home_dir().and_then( |home| {
            home.to_str().and_then( |m_home| {
                let filename = [&m_home, priv_key_path].concat();
                match std::fs::read(&filename) {
                    Ok(mut buffer) => {
                        // todo: normally e.g. with ssh that file should
                        // be parsed, but for simplicity only hard 64 bytes
                        if buffer.len() == 64 {
                            if let Ok(right_formatted_key) = ed25519::Keypair::decode(&mut buffer[..64]) {
                                Some(right_formatted_key)
                            } else {
                                error!("Key file correct format!");
                                None
                            }
                        } else {
                            error!("Key file not 64 byte length!");
                            None
                        }
                    },
                    Err(_) => {
                        // todo: distinguish between io file errors
                        error!("Key file could not be read!");
                        None
                    }
                }
            }).or_else( || {
                error!("Path to key file has illegal symbols!");
                None
            }
            )
        }).or_else(|| {
            error!("Home dir not set!");
            None
        });

        if let Some(good_key) = key_reading_from_file_system {
            good_key
        } else {
            ed25519::Keypair::generate()
        }
    };
}
