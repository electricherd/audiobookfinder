//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.

pub mod tui; // todo: pub is not recommended, I use it for doctest
mod webui;

use self::tui::Tui;
use self::webui::WebUI;
use super::config;

use crate::common::startup::{StartUp, SyncStartUp};

use async_std::{
    sync::{Arc, Mutex},
    task,
};
use libp2p::PeerId;
use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};

type PeerRepresentation = [u8; 16];

/// alive Signal for path from collector
/// or net search alive
#[derive(Clone)]
pub enum CollectionPathAlive {
    BusyPath(usize),
    HostSearch,
}

#[derive(Clone)]
pub enum Status {
    ON,
    OFF,
}

#[derive(Clone)]
pub enum NetMessages {
    Debug,
    ShowNewHost,
    ShowStats { show: NetStats },
}

/// todo: something funny yet .. string is unbounded but like this it worked?
type ForwardNetMessage = (NetMessages, String);

/// internal messages inside ui
pub enum InternalUiMsg {
    Update(ForwardNetMessage),
    StartAnimate(CollectionPathAlive, Status),
    StepAndAnimate(CollectionPathAlive),
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

/// it's just to logically couple these
/// two, because the sync should only
/// happen once and if tui and webui
/// is used it could happen twice
struct SyncWithMain {
    done: bool,
    syncing: Sender<SyncStartUp>,
}
// todo: what about 2 message loops?? web and tui using try_recv?

pub struct Ctrl {
    peer_id: PeerId,
    paths: Vec<String>,
    receiver: Receiver<UiUpdateMsg>,
    with_net: bool,
    sync_main: SyncWithMain,
}

impl Ctrl {
    /// Create a new controller if everything fits.
    ///
    /// # Arguments
    /// * 'peer_id' - The peer_id this client/server uses
    /// * 'paths' - The paths that will be searched
    /// * 'receiver' - The receiver that takes incoming ctrl messages
    /// * 'with_net' - If ctrl should consider net messages
    /// * 'sync_sender' - For start up that main can be informed, "I'm ready"
    pub fn new(
        new_id: PeerId,
        paths: &Vec<String>,
        receiver: Receiver<UiUpdateMsg>,
        with_net: bool,
        sync_sender: Sender<SyncStartUp>,
    ) -> Result<Self, String> {
        Ok(Ctrl {
            peer_id: new_id,
            paths: paths.clone(),
            receiver,
            with_net,
            sync_main: SyncWithMain {
                done: false,
                syncing: sync_sender,
            },
        })
    }

    /// Run the UIs - there is less controlling rather than showing
    pub fn run_tui(&mut self) -> Result<(), String> {
        info!("tui about to run");

        let title = self.peer_id.to_string().clone();
        let paths = self.paths.clone();
        let with_net = self.with_net.clone();

        // set up communication for tui messages
        let (tui_sender, tui_receiver) = channel::<InternalUiMsg>();

        let tui_sender = Arc::new(Mutex::new(tui_sender));
        let internal_tui_sender = tui_sender.clone();

        info!("spawning tui async thread");
        let mut tui = Tui::new(title, &paths, with_net)?;
        task::block_on(async move {
            let tui_sender1 = internal_tui_sender.lock().await.clone();
            let tui_sender2 = tui_sender1.clone();

            // sync/block startup with main thread
            self.sync_with_main();

            info!("spawning tui async thread");
            loop {
                // todo: this below looks like a select! looped async block
                // due to pressing 'q' tui will stop and hence also the loop
                if !tui.refresh().await {
                    break;
                }
                tui.run_cursive(&tui_sender1, &tui_receiver).await;
                if !self.run_message_forwarding(&tui_sender2).await {
                    break;
                }
            }
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
        // sync/block startup with main thread
        self.sync_with_main();

        task::block_on(async move {
            //task::spawn(async move { WebUI::new(peer_representation, net_support) }).await
            WebUI::run(peer_representation, net_support).await
        })
    }

    /// This basically wraps incoming UiUpdateMsg to InternalUiMsg
    /// which kind of defines an extra layer for convenience, and to
    /// be extended and so on.
    async fn run_message_forwarding(&self, forward_sender: &Sender<InternalUiMsg>) -> bool {
        if let Ok(forward_sys_message) = self.receiver.try_recv() {
            match forward_sys_message {
                UiUpdateMsg::NetUpdate((recv_dialog, text)) => {
                    trace!("net update forwarding");
                    forward_sender
                        .send(InternalUiMsg::Update((recv_dialog, text)))
                        .unwrap();
                    true
                }
                UiUpdateMsg::CollectionUpdate(signal, on_off) => {
                    trace!(
                        "forwarding collection message to turn '{}'",
                        match on_off {
                            Status::ON => "on",
                            Status::OFF => "off",
                        }
                    );
                    forward_sender
                        .send(InternalUiMsg::StartAnimate(signal, on_off))
                        .unwrap();
                    true
                }
                UiUpdateMsg::StopUI => {
                    // if error something or Ok(false) results in the same
                    trace!("stop all message forwarding to ui");
                    false
                }
            }
        } else {
            // couldn't find a message yet (trying) but that is fine
            true
        }
    }

    /// Send the sync to main, if not already done
    fn sync_with_main(&mut self) {
        if !self.sync_main.done {
            StartUp::block_on_sync(self.sync_main.syncing.clone(), "ui");
            self.sync_main.done = true;
        } else {
            info!("Sync with main has been already done, shouldn't be a problem!");
        }
    }
}
