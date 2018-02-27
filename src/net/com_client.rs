// taken from trussh example (with corrections)
use std;
use std::sync::Arc;
use std::env;

use futures;
use futures::Future;

use thrussh;
use thrussh::{client, ChannelId, Disconnect};

use thrussh_keys;
use thrussh_keys::key;
use thrussh_keys::load_secret_key;

use config;

#[derive(Clone)]
pub struct ComClient {}

impl client::Handler for ComClient {
    type Error = ();
    type FutureBool = futures::Finished<(Self, bool), Self::Error>;
    type FutureUnit = futures::Finished<Self, Self::Error>;
    type SessionUnit = futures::Finished<(Self, client::Session), Self::Error>;
    type FutureSign = futures::future::FutureResult<(ComClient, thrussh::CryptoVec), Self::Error>;

    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        info!("check_server_key: {:?}", server_public_key);
        futures::finished((self, true))
    }
    fn channel_open_confirmation(
        self,
        channel: ChannelId,
        session: client::Session,
    ) -> Self::SessionUnit {
        debug!("channel_open_confirmation: {:?}", channel);
        futures::finished((self, session))
    }
    fn data(
        self,
        channel: ChannelId,
        ext: Option<u32>,
        data: &[u8],
        session: client::Session,
    ) -> Self::SessionUnit {
        debug!(
            "CLIENT: data on channel {:?} {:?}: {:?}",
            ext,
            channel,
            std::str::from_utf8(data)
        );
        futures::finished((self, session))
    }
}

impl ComClient {
    pub fn run(self, configuration: Arc<client::Config>, _: &str) -> thrussh_keys::Result<()> {
        let key = Self::get_key(
            &config::net::SSH_CLIENT_SEC_KEY_PATH,
            &config::net::SSH_CLIENT_SEC_KEY_PASSWD,
        )?;
        if client::connect(
            config::net::SSH_HOST_AND_PORT,
            configuration,
            None,
            self,
            |connection| {
                connection
                    .authenticate_key(&config::net::SSH_CLIENT_USERNAME, key)
                    .and_then(|session| {
                        session
                            .channel_open_session()
                            .and_then(|(session, channelid)| {
                                session.data(channelid, None, "Hello, world!").and_then(
                                    |(mut session, _)| {
                                        session.disconnect(Disconnect::ByApplication, "Ciao", "");
                                        session
                                    },
                                )
                            })
                    })
            },
        ).is_err() {
            error!("connection could not be established!");
            Err(thrussh_keys::Error::from(thrussh_keys::ErrorKind::Msg(
                "Connection could not be established!".to_string(),
            )))
        } else {
            Ok(())
        }
    }

    fn get_key(priv_key_path: &str, passwd: &str) -> thrussh_keys::Result<key::KeyPair> {
        // home is type changed, so always new ...
        let home = env::home_dir().ok_or(thrussh_keys::Error::from_kind(
            thrussh_keys::ErrorKind::NoHomeDir,
        ))?;
        let home: &str = home.to_str().ok_or(thrussh_keys::Error::from_kind(
            thrussh_keys::ErrorKind::Msg("Path has illegal symbols!".to_string()),
        ))?;
        let file = [&home, priv_key_path].concat();
        if std::fs::File::open(&file).is_ok() {
            load_secret_key(&file, Some(passwd.as_bytes()))
        } else {
            Err(thrussh_keys::Error::from(thrussh_keys::ErrorKind::Msg(
                "KeyFile could not be found!".to_string(),
            )))
        }
    }
}
