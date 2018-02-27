// taken from trussh example (with corrections)
use std;
use std::net;

use futures;

use thrussh;
use thrussh::{server,ChannelId};
use thrussh::server::{Auth, Session};

use thrussh_keys;
use thrussh_keys::key;


#[derive(Clone)]
pub struct ComServer {
    pub name : String
}

impl server::Server for ComServer {
    type Handler = Self;
    fn new(&self, _: net::SocketAddr) -> Self {
        ComServer{
            name : "default".to_string()
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
        debug!(
            "S [{:?}]: data on channel {:?}: {:?}",
            self.name,
            channel,
            std::str::from_utf8(data)
        );
        session.data(channel, None, data); //.unwrap();
        futures::finished((self, session))
    }

}

impl ComServer {
    pub fn create_key_file(_name: &str) -> Result<(), thrussh_keys::Error> {
        match key::KeyPair::generate(key::ED25519) {
            Some(key::KeyPair::Ed25519(..)) => {
                //println!("{:?}",edkey);
            }
            Some(key::KeyPair::RSA{..}) => {
                // to be done
            }
            None => {}
        }
        //
        Ok(())
    }
}
