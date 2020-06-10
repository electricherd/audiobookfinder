//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.

pub mod tui; // todo: pub is not recommended, I use it for doctest
mod webui;

use self::tui::Tui;
use self::webui::WebUI;
use super::config;

use crate::common::startup::{self, StartUp, SyncStartUp};
use crate::ctrl::UiUpdateMsg::NetUpdate;

use async_std::{
    sync::{Arc, Mutex},
    task,
};
use libp2p::PeerId;
use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};

type PeerRepresentation = [u8; 16];

/// Alive Signal for net
#[derive(Clone, Copy)]
pub enum CollectionPathAlive {
    BusyPath(usize),
    HostSearch,
}

#[derive(Copy, Clone)]
pub enum Status {
    ON,
    OFF,
}

#[derive(Clone, Copy)]
pub enum NetMessages {
    Debug,
    ShowNewHost,
    ShowStats { show: NetStats },
}

type ForwardNetMessage = (NetMessages, String);

pub enum InternalUiMsg {
    Update(ForwardNetMessage),
    Animate(CollectionPathAlive, Status),
    TimeOut(CollectionPathAlive),
}

#[derive(Clone)]
pub enum UiUpdateMsg {
    NetUpdate(ForwardNetMessage),
    CollectionUpdate(CollectionPathAlive, Status),
    StopUI,
}

#[derive(Copy, Clone)]
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
    pub fn run_tui(
        &mut self,
        ready_sender: Sender<SyncStartUp>,
        external_receiver: Receiver<UiUpdateMsg>,
    ) -> Result<(), String> {
        info!("tui about to run");

        let title = self.peer_id.to_string().clone();
        let paths = self.paths.clone();
        let with_net = self.with_net.clone();

        // set up communication for tui messages
        let (tui_sender, tui_receiver) = channel::<InternalUiMsg>();

        let tui_sender = Arc::new(Mutex::new(tui_sender));
        let internal_tui_sender = tui_sender.clone();

        // todo: see where
        StartUp::block_on_sync(ready_sender, "ui");

        // loop external messages and forward to internal
        // ui messages
        let tui_spawner = task::block_on(async move {
            let mut tui;
            info!("spawning tui async thread");
            let tui_sender = internal_tui_sender.lock().await.clone();
            tui = Tui::new(title, &paths, with_net)?;
            tui.run(tui_receiver, tui_sender).await;
            Ok::<bool, String>(true)
        });

        let message_spawner = task::block_on(async move {
            info!("spawning tui async thread");
            Self::run_message_forwarding(external_receiver, tui_sender.lock().await.clone()).await;
        });
        Ok(())
    }

    /// Run the controller
    pub fn run_webui(&mut self) -> io::Result<()> {
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

    async fn run_message_forwarding(
        from_external_receiver: Receiver<UiUpdateMsg>,
        forward_sender: Sender<InternalUiMsg>,
    ) -> Result<bool, String> {
        info!("entering message to tui forwarding");

        let mut status = true;
        if let Ok(forward_sys_message) = from_external_receiver.recv() {
            match forward_sys_message {
                UiUpdateMsg::NetUpdate((recv_dialog, text)) => {
                    info!("net update");
                    forward_sender
                        .send(InternalUiMsg::Update((recv_dialog, text)))
                        .unwrap();
                }
                UiUpdateMsg::CollectionUpdate(signal, on_off) => {
                    info!("collection update");
                    forward_sender
                        .send(InternalUiMsg::Animate(signal, on_off))
                        .unwrap();
                }
                UiUpdateMsg::StopUI => {
                    // if error something or Ok(false) results in the same
                    status = false;
                }
            }
        }
        Ok::<bool, String>(status)
    }
} // impl Controller
