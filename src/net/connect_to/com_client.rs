//! The ssh client yet of what it will be capable of
//! and taken from trussh example (with corrections).

use super::super::{
    config,
    connect_to::sc_com_to::{self, SCClient},
    data::{DataAuth, DataSession},
    key_keeper,
};
use bincode;
use futures::{self, Future};
use libp2p::PeerId;
use std::{net::IpAddr, sync::Arc};
use thrussh;
use thrussh_keys;
use tokio_io;

#[derive(Clone)]
pub struct ComClient {
    peer_id: PeerId,
}

impl thrussh::client::Handler for ComClient {
    type Error = ();
    type FutureBool = futures::Finished<(Self, bool), Self::Error>;
    type FutureUnit = futures::Finished<Self, Self::Error>;
    type SessionUnit = futures::Finished<(Self, thrussh::client::Session), Self::Error>;
    type FutureSign = futures::future::FutureResult<(ComClient, thrussh::CryptoVec), Self::Error>;

    fn check_server_key(
        self,
        _server_public_key: &thrussh_keys::key::PublicKey,
    ) -> Self::FutureBool {
        futures::finished((self, true))
    }
    fn channel_open_confirmation(
        self,
        channel: thrussh::ChannelId,
        session: thrussh::client::Session,
    ) -> Self::SessionUnit {
        debug!("channel_open_confirmation: {:?}", channel);
        futures::finished((self, session))
    }
    fn data(
        self,
        channel: thrussh::ChannelId,
        ext: Option<u32>,
        data: &[u8],
        session: thrussh::client::Session,
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
    pub fn new(peer_id: PeerId) -> ComClient {
        ComClient { peer_id: peer_id }
    }

    pub fn run(self, configuration: Arc<thrussh::client::Config>, ip_addr: &IpAddr) {
        let id = self.peer_id.clone();

        // just use a copy to arc
        let key = Arc::clone(&*key_keeper::SERVER_KEY_SSH);
        //
        // start the state machine
        //
        // toDo: safe this here with an assert or so
        let mut _sm = sc_com_to::StateMachine::new(SCClient {});

        let _ = thrussh::client::connect_future(
            (*ip_addr, config::net::PORT_SSH),
            configuration,
            None,
            self,
            |connection| {
                // tokio I assume starts within
                info!("Key file, password ok!");

                connection
                    .authenticate_key(&config::net::SSH_CLIENT_USERNAME, key)
                    .or_else(|e| {
                        error!("Authentification didn't work!");
                        Err(e)
                    })
                    .and_then(|valid_session| Self::continue_session(id, valid_session))
                    .or_else(|e| {
                        error!("Session could not be created!");
                        Err(e)
                    })
            },
        )
        .or_else(|_e| {
            error!(
                "Connection with {:?}:{:?} could not be established!",
                ip_addr,
                config::net::PORT_SSH
            );
            Err(thrussh_keys::Error::IO(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Connection could not be established!",
            )))
        });
        info!("run done ......................");
    }

    fn continue_session<R, H>(
        peer_id: PeerId,
        connection: thrussh::client::Connection<R, H>,
    ) -> impl Future<Item = (), Error = thrussh::HandlerError<<H as thrussh::client::Handler>::Error>>
    where
        R: tokio_io::AsyncRead + tokio_io::AsyncWrite + thrussh::Tcp,
        H: thrussh::client::Handler,
    {
        info!("Session could be established!");
        connection
            .channel_open_session()
            .and_then(move |(session, channelid)| {
                info!("Session could be opened, sending out!");

                // send data
                let datagram = Self::get_data(&peer_id);

                Self::send(channelid, datagram, session)
            })
            .or_else(|e| {
                error!("Channel could not be openend!");
                Err(e)
            })
    }

    fn send<R, H>(
        channelid: thrussh::ChannelId,
        to_send_data: DataSession,
        connection: thrussh::client::Connection<R, H>,
    ) -> impl Future<Item = (), Error = thrussh::HandlerError<<H as thrussh::client::Handler>::Error>>
    where
        R: tokio_io::AsyncRead + tokio_io::AsyncWrite + thrussh::Tcp,
        H: thrussh::client::Handler,
    {
        connection
            .data(channelid, None, bincode::serialize(&to_send_data).unwrap())
            .and_then(|(mut session, _)| {
                session.disconnect(thrussh::Disconnect::ByApplication, "Ciao", "");
                //session
                futures::finished(())
            })
            .or_else(|e| {
                error!("Session could not be opened!");
                Err(e)
            })
    }

    fn get_data(peer_id: &PeerId) -> DataSession {
        // depending on what you want, so far only auth
        DataSession::Auth {
            auth: DataAuth::new((*peer_id).clone()),
        }
    }
}
