//! The ssh client yet of what it will be capable of
//! and taken from trussh example (with corrections).
use bincode;
use config;
use futures::{self, Future};
use net::data::{DataAuth, DataSession};
use std::{self, env, net::IpAddr, sync::Arc};
use thrussh::{self, client, ChannelId, Disconnect};
use thrussh_keys::{self, key, load_secret_key};

#[derive(Clone)]
pub struct ComClient {
    uuid: String,
}

impl client::Handler for ComClient {
    type Error = ();
    type FutureBool = futures::Finished<(Self, bool), Self::Error>;
    type FutureUnit = futures::Finished<Self, Self::Error>;
    type SessionUnit = futures::Finished<(Self, client::Session), Self::Error>;
    type FutureSign = futures::future::FutureResult<(ComClient, thrussh::CryptoVec), Self::Error>;

    fn check_server_key(self, _server_public_key: &key::PublicKey) -> Self::FutureBool {
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
        let res_session: Result<DataSession, _> = bincode::deserialize(&data[..]);
        if let Ok(work_session) = res_session {
            match work_session {
                DataSession::Auth { auth } => {
                    info!(
                        "CLIENT: data on channel {:?} {:?}: {:?}",
                        ext,
                        channel,
                        auth.get_id()
                    );
                }
                DataSession::Data { .. } => {}
            }
        }
        futures::finished((self, session))
    }
}

impl ComClient {
    pub fn new(uuid_name: String) -> ComClient {
        ComClient { uuid: uuid_name }
    }

    pub fn run(
        self,
        configuration: Arc<client::Config>,
        ip_addr: IpAddr,
    ) -> thrussh_keys::Result<()> {
        let id = self.uuid.clone();
        Self::get_key(
            &config::net::SSH_CLIENT_SEC_KEY_PATH,
            &config::net::SSH_CLIENT_SEC_KEY_PASSWD,
        ).and_then(|key| {
            client::connect(
                (ip_addr, config::net::SSH_PORT),
                configuration,
                None,
                self,
                |connection| {
                    info!("Key file, password ok!");
                    connection
                        .authenticate_key(&config::net::SSH_CLIENT_USERNAME, key)
                        .or_else(|e| {
                            error!("Authentification didn't work!");
                            Err(e)
                        })
                        .and_then(|session| {
                            info!("Session could be established!");
                            session
                                .channel_open_session()
                                .and_then(|(session, channelid)| {
                                    info!("Session could be opened, sending out!");

                                    // send real authentification
                                    let auth_data = DataSession::Auth {
                                        auth: DataAuth::new(id),
                                    };

                                    session
                                        .data(
                                            channelid,
                                            None,
                                            bincode::serialize(&auth_data).unwrap(),
                                        )
                                        .and_then(|(mut session, _)| {
                                            session.disconnect(
                                                Disconnect::ByApplication,
                                                "Ciao",
                                                "",
                                            );
                                            session
                                        })
                                        .or_else(|e| {
                                            error!("Session could not be opened!");
                                            Err(e)
                                        })
                                })
                                .or_else(|e| {
                                    error!("Channel could not be openend!");
                                    Err(e)
                                })
                        })
                        .or_else(|e| {
                            error!("Session could not be created!");
                            Err(e)
                        })
                },
            ).or_else(|_e| {
                error!(
                    "Connection with {:?}:{:?} could not be established!",
                    ip_addr,
                    config::net::SSH_PORT
                );
                Err(thrussh_keys::Error::from(thrussh_keys::ErrorKind::Msg(
                    "Connection could not be established!".to_string(),
                )))
            })
        })
            .or_else(|e| {
                // error!("Key file: {:?}!", e);
                error!("Key file error!");
                Err(e)
            })
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
            error!("Not found or password wrong: {:?}", &file);
            Err(thrussh_keys::Error::from(thrussh_keys::ErrorKind::Msg(
                "KeyFile could not be found!".to_string(),
            )))
        }
    }
}
