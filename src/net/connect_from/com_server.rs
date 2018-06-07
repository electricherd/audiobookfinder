//! The ssh communication server, decisions will be made here, of how to interact with
//! the clients, is basically still taken from trussh example (with corrections)
use bincode;
use futures;
use std::{self, net};
use thrussh::{
    self, server::{self, Auth, Session}, ChannelId,
};
use thrussh_keys::{self, key};

use super::super::data::DataSession;

#[derive(Clone)]
pub struct ComServer {
    pub name: String,
    pub connector: Option<net::SocketAddr>,
}

impl server::Server for ComServer {
    type Handler = Self;
    fn new(&self, connector: net::SocketAddr) -> Self {
        ComServer {
            name: self.name.clone(),
            connector: Some(connector),
        }
    }
}

impl thrussh::server::Handler for ComServer {
    type Error = ();
    type FutureAuth = futures::Finished<(Self, server::Auth), Self::Error>;
    type FutureUnit = futures::Finished<(Self, server::Session), Self::Error>;
    type FutureBool = futures::Finished<(Self, server::Session, bool), Self::Error>;

    fn finished_auth(self, auth: Auth) -> Self::FutureAuth {
        futures::finished((self, auth))
    }
    fn finished_bool(self, session: Session, b: bool) -> Self::FutureBool {
        futures::finished((self, session, b))
    }
    fn finished(self, session: Session) -> Self::FutureUnit {
        futures::finished((self, session))
    }

    fn auth_publickey(self, _: &str, _: &key::PublicKey) -> Self::FutureAuth {
        futures::finished((self, server::Auth::Accept))
    }
    fn data(
        self,
        channel: ChannelId,
        data: &[u8],
        mut session: server::Session,
    ) -> Self::FutureUnit {
        let session_data: Result<DataSession, _> = bincode::deserialize_from(data);
        match session_data {
            Ok(deserialized) => match deserialized {
                DataSession::Auth { ref auth } => {
                    let client_id = auth.get_id();
                    let client_version = auth.get_version();
                    info!(
                        "Srv[{:?}]: auth from channel {:?}: with id {:?} and version {:?}",
                        self.name,
                        channel,
                        client_id,
                        client_version
                    );
                }
                DataSession::Data { .. } => {
                    info!(
                        "Srv[{:?}]: data from channel {:?}: {:?}",
                        self.name,
                        channel,
                        std::str::from_utf8(data)
                    );
                }
            },
            Err(_) => {
                info!(
                    "Srv[{:?}]: not valid session data on channel {:?}: {:?}",
                    self.name,
                    channel,
                    std::str::from_utf8(data)
                );
            }
        }
        session.data(channel, None, data);
        futures::finished((self, session))
    }
}

impl ComServer {
    pub fn create_key_file(_name: &str) -> Result<(), thrussh_keys::Error> {
        match key::KeyPair::generate(key::ED25519) {
            Some(key::KeyPair::Ed25519(..)) => {
                //println!("{:?}",edkey);
            }
            Some(key::KeyPair::RSA { .. }) => {
                // to be done
            }
            None => {}
        }
        //
        Ok(())
    }
}