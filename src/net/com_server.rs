// taken from trussh example (with corrections)
use thrussh;
use futures;

use thrussh::*;
use thrussh::server::{Auth, Session};
use thrussh_keys::*;

use std;
use std::net;

#[derive(Clone)]
pub struct ComServer {}

impl server::Server for ComServer {
    type Handler = Self;
    fn new(&self, _: net::SocketAddr) -> Self {
        ComServer{}
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
        println!(
            "data on channel {:?}: {:?}",
            channel,
            std::str::from_utf8(data)
        );
        session.data(channel, None, data); //.unwrap();
        futures::finished((self, session))
    }
}
