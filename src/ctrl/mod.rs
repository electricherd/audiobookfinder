//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.

pub mod tui; // todo: pub is not recommended, I use it for doctest
mod webui;

use self::tui::Tui;
use self::webui::WebUI;
use super::config;
use libp2p::PeerId;
use std::io;
use std::sync::mpsc::{Receiver, Sender};

type PeerRepresentation = [u8; 16];
/// Alive Signal for net
#[derive(Clone)]
pub enum NetAlive {
    BusyPath(usize),
    HostSearch,
}

pub enum Status {
    ON,
    OFF,
}

pub enum ReceiveDialog {
    Debug,
    ShowNewHost,
    ShowStats { show: NetStats },
}

pub enum UiMsg {
    Update(ReceiveDialog, String),
    Animate(NetAlive, Status),
    TimeOut(NetAlive),
}

pub enum SystemMsg {
    Update(ReceiveDialog, String),
    StartAnimation(NetAlive, Status),
}

pub struct NetStats {
    pub line: usize,
    pub max: usize,
}

pub struct Ctrl {
    peer_id: PeerId,
    paths: Vec<String>,
    receiver: Receiver<SystemMsg>,
    sender: Sender<SystemMsg>,
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
        receiver: Receiver<SystemMsg>,
        sender: Sender<SystemMsg>,
        with_net: bool,
    ) -> Result<Self, String> {
        Ok(Ctrl {
            peer_id: new_id,
            paths: paths.clone(),
            receiver,
            sender,
            with_net,
        })
    }
    /// Run the controller
    pub async fn run_tui(&mut self) -> Result<(), String> {
        let mut tui = Tui::new(
            self.peer_id.to_string(),
            self.sender.clone(),
            &self.paths,
            self.with_net,
        )?;

        while tui.step() {
            while let Some(forward_sys_message) = self.receiver.try_iter().next() {
                // Handle messages arriving from the UI.
                println!("incoming ....");
                match forward_sys_message {
                    SystemMsg::Update(recv_dialog, text) => {
                        println!("Update");
                        tui.ui_sender
                            .send(UiMsg::Update(recv_dialog, text))
                            .unwrap();
                    }
                    SystemMsg::StartAnimation(signal, on_off) => {
                        println!("Start Animation");
                        tui.ui_sender.send(UiMsg::Animate(signal, on_off)).unwrap();
                    }
                };
            }
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
