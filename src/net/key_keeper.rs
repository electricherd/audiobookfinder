//! A component to use key

use std::{self, env};
use thrussh_keys::{self, key, load_secret_key};

use super::super::config;

pub fn get_server_key<'a>() ->  &'a Option<key::KeyPair> {
    &*SERVER_KEY
}

pub fn is_good() -> bool {
    (*SERVER_KEY).is_some()
}

lazy_static!{
    static ref SERVER_KEY : Option<key::KeyPair> = {
        let priv_key_path = config::net::SSH_CLIENT_SEC_KEY_PATH;
        let passwd = config::net::SSH_CLIENT_SEC_KEY_PASSWD;

        // home is type changed, so always new ...
        let static_inside = env::home_dir().and_then( |home| {
            home.to_str().and_then( |m_home| {
                let file = [&m_home, priv_key_path].concat();
                let result = if std::fs::File::open(&file).is_ok() {
                    load_secret_key(&file, Some(passwd.as_bytes()))
                } else {
                    error!("Not found or password wrong: {:?}", &file);
                    Err(thrussh_keys::Error::from(thrussh_keys::ErrorKind::Msg(
                        "KeyFile could not be found!".to_string(),
                    )))
                };
                match result {
                    Ok(good) => Some(good),
                    Err(_) => None
                }
            }).or_else( || {
                // toDo: correct this here ... traces my dear
                //thrussh_keys::Error::from_kind(thrussh_keys::ErrorKind::Msg("Path has illegal symbols!".to_string()));
                None
            }
            )
        }).or_else(|| {
            //thrussh_keys::Error::from_kind(
            //thrussh_keys::ErrorKind::NoHomeDir)
            None
        });
        static_inside
    };
}
