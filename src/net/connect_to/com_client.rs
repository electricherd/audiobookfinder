//! The ssh client yet of what it will be capable of
//! and taken from trussh example (with corrections).

use bincode;
use futures::{self, Future};
use std::{net::IpAddr, sync::Arc};
use thrussh::{self, client, ChannelId, Disconnect};
use thrussh_keys::{self, key};

use super::super::{
    config, connect_to::sc_com_to::{SCClient, SCClientFuture}, data::{DataAuth, DataSession},
    key_keeper,
};

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

    pub fn run(self, configuration: Arc<client::Config>, ip_addr: &IpAddr) {
        let id = self.uuid.clone();
        //
        // start the state machine
        //
        // toDo: safe this here with an assert or so
        let sc_future: SCClientFuture = SCClient::start();

        let _ = client::connect(
            (*ip_addr, config::net::SSH_PORT),
            configuration,
            None,
            self,
            |connection| {
                // tokio I assume starts within
                info!("Key file, password ok!");

                let static_server_key_been_already_tested_as_good = key_keeper::get_server_key();
                assert!(
                    static_server_key_been_already_tested_as_good.is_some(),
                    "Key code should have been tested before, so it should never reach this point."
                );
                let key = static_server_key_been_already_tested_as_good.unwrap();

                let further = connection.authenticate_key(&config::net::SSH_CLIENT_USERNAME, key);
                // split
                let even_further = further.or_else(|e| {
                    error!("Authentification didn't work!");
                    Err(e)
                });
                even_further
                    .and_then(|session| {
                        info!("Session could be established!");
                        let more_further = session.channel_open_session();
                        more_further
                            .and_then(|(session, channelid)| {
                                info!("Session could be opened, sending out!");

                                // send real authentification
                                let auth_data = DataSession::Auth {
                                    auth: DataAuth::new(id),
                                };

                                session
                                    .data(channelid, None, bincode::serialize(&auth_data).unwrap())
                                    .and_then(|(mut session, _)| {
                                        session.disconnect(Disconnect::ByApplication, "Ciao", "");
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
        });
        info!("run done ......................");
    }
}
