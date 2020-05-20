//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.

pub mod tui; // todo: pub is not recommended, I use it for doctest
mod webui;

use self::tui::Tui;
use self::webui::WebUI;
use super::config;
use async_std::task;
use libp2p::PeerId;
use std::io::{self, Error, ErrorKind};
use std::sync::mpsc::{Receiver, Sender};

type PeerRepresentation = [u8; 16];

#[derive(Clone)]
pub enum Alive {
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
    Animate(Alive, Status),
    TimeOut(Alive),
}

pub enum SystemMsg {
    Update(ReceiveDialog, String),
    StartAnimation(Alive, Status),
}

pub struct NetStats {
    pub line: usize,
    pub max: usize,
}

pub struct Ctrl {
    rx: Receiver<SystemMsg>,
    ui: Tui,
    peer_id: PeerId,
    with_net: bool,
}

impl Ctrl {
    /// Create a new controller if everything fits.
    ///
    /// # Arguments
    /// * 'uuid' - The uuid this client/server uses
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
        let c_ui = Tui::new(new_id.to_string(), sender.clone(), &paths, with_net)?;

        Ok(Ctrl {
            rx: receiver,
            ui: c_ui,
            peer_id: new_id,
            with_net,
        })
    }
    /// Run the controller
    pub fn run_tui(&mut self) {
        while self.ui.step() {
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    SystemMsg::Update(recv_dialog, text) => {
                        self.ui
                            .ui_sender
                            .send(UiMsg::Update(recv_dialog, text))
                            .unwrap();
                    }
                    SystemMsg::StartAnimation(signal, on_off) => {
                        self.ui
                            .ui_sender
                            .send(UiMsg::Animate(signal, on_off))
                            .unwrap();
                    }
                };
            }
        }
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
        task::spawn(async move {
            WebUI::new(peer_representation, net_support).unwrap();
        });
        Ok(())
    }
} // impl Controller
