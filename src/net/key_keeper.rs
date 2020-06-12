//! A component to use secure communication
//! using keys. It is not yet clear ...
//! basically what libp2p offers is best.

use super::super::config;
use dirs;
use libp2p::{
    self,
    identity::{
        ed25519::{self, PublicKey},
        Keypair,
        PublicKey::Ed25519,
    },
    PeerId,
};
use std::{
    io::{Error, ErrorKind},
    path::Path,
};

pub fn get_p2p_server_public_key<'a>() -> PublicKey {
    (&*SERVER_KEY).public()
}

pub fn get_p2p_server_id<'a>() -> PeerId {
    PeerId::from_public_key(Ed25519(get_p2p_server_public_key()))
}

lazy_static! {
    pub static ref SERVER_KEY : ed25519::Keypair = {
        let priv_key_path = config::net::PEER_SEC_KEY_PATH;
        let priv_key_file = config::net::PEER_SEC_KEY_FILE;

        // home is type changed, so always new ...
        // todo: looks awful but this will stay as something to look up for result
        //       option handling (even the rustfmt formatter is on strike)!
        let key_reading_from_file_system =
            dirs::home_dir()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Home dir not set!"))
            .and_then( |home| {
                home.to_str()
                .ok_or_else(|| Error::new(ErrorKind::Other, "Path to key file has illegal symbols!"))
                .and_then(|m_home| {
                    let full_filename = Path::new(m_home).join(priv_key_path).join(priv_key_file);
                    match std::fs::read(&full_filename) {
                        Ok(mut buffer) => {
                            // todo: normally e.g. with ssh that file should
                            //       be parsed, but for simplicity only hard 64 bytes
                            if buffer.len() == 64 {
                                if let Ok(right_formatted_key) =
                                        ed25519::Keypair::decode(&mut buffer[..64]) {
                                    Result::Ok(right_formatted_key)
                                } else {
                                    Err(Error::new(ErrorKind::Other, "Key file correct format!"))
                                }
                            } else {
                                Err(Error::new(ErrorKind::Other, "Key file not 64 byte length!"))
                            }
                        },
                        Err(error) => {
                            let error_text = format!("Key file could not be read: {:?} ",
                                                      error);
                            Err(Error::new(ErrorKind::Other, error_text))
                        }
                    }
                }
            )
        });

        key_reading_from_file_system.unwrap_or_else(|error| {
            info!("{:?}",error.to_string());
            info!("Creating an own key since no good one existed!");
            let new_key = ed25519::Keypair::generate();
            // todo: store it then or when finishing by signing it
            // let secret_key = &STATIC_SECRET;
            // secret_key.sign(new_key).and_then(|signed_key| {
            //      info!("_{:?}_", signed_key);
            // });
            new_key
        })
    };

    static ref STATIC_SECRET: Keypair = {
        // of course, and especially in a git project you should not use a public ;-)
        // secret key, but it is for testing
        let mut key_as_buffer = *include_bytes!("static_secret.pk8");
        let keypair = Keypair::rsa_from_pkcs8(&mut key_as_buffer);
        keypair.expect("No this should not happen, key file must be valid in GIT")
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    //run "cargo test -- --nocapture" to see debug println
    #[test]
    fn verify_is_correct_in_file_system_static_secret() {
        // just call the function
        let _keypair: &Keypair = &STATIC_SECRET;
        assert!(true);
    }
}
