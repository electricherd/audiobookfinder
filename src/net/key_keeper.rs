//! A component to use key

use dirs;
use std::{self};
use thrussh;
use thrussh_keys::{decode_secret_key, key};

use super::super::config;

pub fn get_server_key<'a>() -> Option<key::KeyPair> {
    let passwd = config::net::SSH_CLIENT_SEC_KEY_PASSWD;
    let copied = (&*SERVER_KEY).clone();
    copied.and_then(|key_slice| {
        key_slice
            .get(..)
            .and_then(|sliced| match std::str::from_utf8(sliced) {
                Ok(valid_utf8) => match decode_secret_key(valid_utf8, Some(passwd.as_bytes())) {
                    Ok(good) => Some(good),
                    Err(_) => None,
                },
                Err(_) => None,
            })
    })
}

pub fn is_good() -> bool {
    (*SERVER_KEY).is_some()
}

lazy_static!{
    static ref SERVER_KEY : Option<thrussh::CryptoVec> = {
        let priv_key_path = config::net::SSH_CLIENT_SEC_KEY_PATH;

        // home is type changed, so always new ...
        dirs::home_dir().and_then( |home| {
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
        })
    };
}
