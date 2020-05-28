//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.

pub mod tui; // todo: pub is not recommended, I use it for doctest
mod webui;

use self::tui::Tui;
use self::webui::WebUI;
use super::config;
use async_std::sync;
use futures::{future::FutureExt, join, pin_mut, select};
use libp2p::PeerId;
use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};

type PeerRepresentation = [u8; 16];
/// Alive Signal for net
#[derive(Clone)]
pub enum CollectionPathAlive {
    BusyPath(usize),
    HostSearch,
}

pub enum Status {
    ON,
    OFF,
}

pub enum NetMessages {
    Debug,
    ShowNewHost,
    ShowStats { show: NetStats },
}

pub enum InternalUiMsg {
    Update(NetMessages, String),
    Animate(CollectionPathAlive, Status),
    TimeOut(CollectionPathAlive),
}

pub enum UiUpdateMsg {
    NetUpdate(NetMessages, String),
    CollectionUpdate(CollectionPathAlive, Status),
    StopUI,
}

pub struct NetStats {
    pub line: usize,
    pub max: usize,
}

pub struct Ctrl {
    peer_id: PeerId,
    paths: Vec<String>,
    sender: Sender<UiUpdateMsg>,
    with_net: bool,
}

impl Ctrl {
    /// Create a new controller if everything fits.
    ///
    /// # Arguments
    /// * 'peer_id' - The peer_id this client/server uses
    /// * 'paths' - The paths that will be searched
    /// * 'receiver' - The receiver that takes incoming ctrl messages
    /// * 'sender'   - The sender that sends from ctrl
    /// * 'with_net' - If ctrl should consider net messages
    pub fn new(
        new_id: PeerId,
        paths: &Vec<String>,
        sender: Sender<UiUpdateMsg>,
        with_net: bool,
    ) -> Result<Self, String> {
        Ok(Ctrl {
            peer_id: new_id,
            paths: paths.clone(),
            sender,
            with_net,
        })
    }
    /// Run the controller
    pub async fn run_tui(&mut self, receiver: Receiver<UiUpdateMsg>) -> Result<(), String> {
        info!("tui about to run");

        let mut tui = Tui::new(
            self.peer_id.to_string().clone(),
            self.sender.clone(),
            &self.paths.clone(),
            self.with_net.clone(),
        );

        if let Ok(mut working_tui) = tui {
            info!("tui could be initialized");
            working_tui
                .run(
                    receiver,
                    self.sender.clone(),
                    self.peer_id.to_string().clone(),
                    self.paths.clone(),
                    self.with_net.clone(),
                )
                .await;
        }
        Ok(())
    }

    /// Run the controller
    pub async fn run_webui(&mut self) -> io::Result<()> {
        let net_support = self.with_net;
        // todo: damn, please make this nice if you can
        let mut peer_representation: PeerRepresentation = [0 as u8; 16];
        peer_representation.copy_from_slice(&self.peer_id.as_bytes()[..16]);
        if webbrowser::open(&["http://", config::net::WEBSOCKET_ADDR].concat()).is_err() {
            info!("Could not open browser!");
        }
        //task::spawn(async move { WebUI::new(peer_representation, net_support) }).await
        WebUI::new(peer_representation, net_support).and_then(|_webui| {
            // _webui is good and see what we can do
            Ok(())
        })
    }
} // impl Controller
