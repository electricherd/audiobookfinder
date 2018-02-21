// taken from trussh example (with corrections)
use futures;
use futures::Future;

use thrussh;
use thrussh::*;
use thrussh_keys::*;

use std;
use std::io::Read;
use std::sync::Arc;

use config;

#[derive(Clone)]
struct ComClient {}

impl client::Handler for ComClient {
    type Error = ();
    type FutureBool = futures::Finished<(Self, bool), Self::Error>;
    type FutureUnit = futures::Finished<Self, Self::Error>;
    type SessionUnit = futures::Finished<(Self, client::Session), Self::Error>;
    type FutureSign = futures::future::FutureResult<(ComClient, thrussh::CryptoVec), Self::Error>;

    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        println!("check_server_key: {:?}", server_public_key);
        futures::finished((self, true))
    }
    fn channel_open_confirmation(
        self,
        channel: ChannelId,
        session: client::Session,
    ) -> Self::SessionUnit {
        println!("channel_open_confirmation: {:?}", channel);
        futures::finished((self, session))
    }
    fn data(
        self,
        channel: ChannelId,
        ext: Option<u32>,
        data: &[u8],
        session: client::Session,
    ) -> Self::SessionUnit {
        println!(
            "data on channel {:?} {:?}: {:?}",
            ext,
            channel,
            std::str::from_utf8(data)
        );
        futures::finished((self, session))
    }
}

impl ComClient {
    fn run(self, configuration: Arc<client::Config>, _: &str) {
        match std::fs::File::open(config::net::SSH_CLIENT_KEY_FILE) {
            Ok(mut key_file) => {
                client::connect(
                    config::net::SSH_HOST_AND_PORT,
                    configuration,
                    None,
                    self,
                    |connection| {
                        let mut key = String::new();
                        key_file.read_to_string(&mut key).unwrap();
                        let key = load_secret_key(&key, None).unwrap();

                        connection
                            .authenticate_key(config::net::SSH_CLIENT_USERNAME, key)
                            .and_then(|session| {
                                session
                                    .channel_open_session()
                                    .and_then(|(session, channelid)| {
                                        session.data(channelid, None, "Hello, world!").and_then(
                                            |(mut session, _)| {
                                                session.disconnect(
                                                    Disconnect::ByApplication,
                                                    "Ciao",
                                                    "",
                                                );
                                                session
                                            },
                                        )
                                    })
                            })
                    },
                ).unwrap();
            }
            Err(..) => {
                println!(
                    "SSH client key file '{:?}' could not be found!!",
                    config::net::SSH_CLIENT_KEY_FILE
                );
            }
        }
    }
}
