// taken from trussh example (with corrections)
use futures;
use futures::Future;

use thrussh;
use thrussh::*;
use thrussh_keys::key;
use thrussh_keys::{Result,load_secret_key,decode_secret_key};

use std;
use std::io::Read;
use std::sync::Arc;

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
            "CLIENT: data on channel {:?} {:?}: {:?}",
            ext,
            channel,
            std::str::from_utf8(data)
        );
        futures::finished((self, session))
    }
}

impl ComClient {
    pub fn run(self, configuration: Arc<client::Config>, _: &str) -> Result<()> {
        //match std::fs::File::open(config::net::SSH_CLIENT_KEY_FILE) {
            //Ok(mut key_file) => {
            //    let mut key = String::new();
            //    if key_file.read_to_string(&mut key).is_err() {
            //        println!("not found {:?}", key_file);
            //    } else {
                    //if let Ok(key) = load_secret_key(&key, Some(b"b")) {
                    if let Ok(key) = decode_secret_key(config::net::SSH_CLIENT_SEC_KEY, Some(b"blabla")) {
                        if client::connect(
                            config::net::SSH_HOST_AND_PORT,
                            configuration,
                            None,
                            self,
                            |connection| {
                                connection
                                    .authenticate_key(config::net::SSH_CLIENT_USERNAME, key)
                                    .and_then(|session| {
                                        session.channel_open_session().and_then(
                                            |(session, channelid)| {
                                                session
                                                    .data(channelid, None, "Hello, world!")
                                                    .and_then(|(mut session, _)| {
                                                        session.disconnect(
                                                            Disconnect::ByApplication,
                                                            "Ciao",
                                                            "",
                                                        );
                                                        session
                                                    })
                                            },
                                        )
                                    })
                            },
                        ).is_err() {
                            println!("connection could not be established!");
                        }
                    } else {
                        println!("secret key not good");
                    }
                //}
            //}
            //Err(..) => {
            //    println!(
            //        "SSH client key file '{:?}' could not be found!!",
            //        config::net::SSH_CLIENT_KEY_FILE
            //    );
            //}
        //}
        Ok(())
    }
}
