//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.

pub mod tui; // todo: pub is not recommended, I use it for doctest
mod webui;

use self::tui::Tui;
use self::webui::WebUI;
use super::config;
use crate::common::startup::{StartUp, SyncStartUp};
use crate::ctrl::webui::ChannelForwarder;
use async_std::task;
use libp2p::PeerId;
use std::{
    io,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

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
        with_net: bool,
        sync_sender: Sender<SyncStartUp>,
    ) -> Self {
        Self {
            peer_id: new_id,
            paths: paths.clone(),
            with_net,
            sync_main: SyncWithMain {
                done: false,
                syncing: sync_sender,
            },
        }
    }

    pub fn run(
        new_id: PeerId,
        paths: &Vec<String>,
        receiver: Receiver<UiUpdateMsg>,
        with_net: bool,
        sync_sender: Sender<SyncStartUp>,
        has_webui: bool,
        has_tui: bool,
    ) -> Result<(), std::io::Error> {
        let instance = Ctrl::new(new_id, paths, with_net, sync_sender);

        let arc_self_tui = Arc::new(Mutex::new(instance));
        let arc_self_webui = arc_self_tui.clone();
        let arc_self_message_loop = arc_self_tui.clone();

        // all senders that UiUpdateMessages will be forwarded to
        let mut internal_senders: Vec<Sender<InternalUiMsg>> = vec![];

        // 1) tui thread
        let thread_tui = if has_tui {
            let (sender_to_register, receiver_to_tui_thread) = channel::<InternalUiMsg>();
            let resender = sender_to_register.clone();
            internal_senders.push(sender_to_register);
            Self::spawn_tui(arc_self_tui, resender, receiver_to_tui_thread)?
        } else {
            // empty thread
            std::thread::spawn(|| Ok(()))
        };

        // 2) webui thread
        let thread_webui = if has_webui {
            let (sender_to_register, receiver_to_web_ui_thread) = channel::<InternalUiMsg>();
            internal_senders.push(sender_to_register);
            Self::spawn_webui(arc_self_webui, receiver_to_web_ui_thread)?
        } else {
            // empty thread
            std::thread::spawn(|| Ok(()))
        };

        // sync/block startup with main thread
        {
            let mut unlocker = arc_self_message_loop.lock().unwrap();
            unlocker.sync_with_main();
        }

        // 3) message ui forwarding thread
        let message_loop = Self::spawn_message_loop(receiver, internal_senders);

        // todo: yes ... way to go!!!
        //drop(thread_tui);
        let res_tui = thread_tui.join();
        // todo: there is no quitting yet ... is there???
        drop(thread_webui);
        drop(message_loop);
        Ok::<(), std::io::Error>(())
    }

    fn spawn_webui(
        this: Arc<Mutex<Self>>,
        receiver: Receiver<InternalUiMsg>,
    ) -> Result<thread::JoinHandle<Result<(), std::io::Error>>, std::io::Error> {
        let mut peer_representation: PeerRepresentation = [0 as u8; 16];
        let with_net;
        {
            let unlocker = this.lock().unwrap();
            peer_representation.copy_from_slice(&unlocker.peer_id.as_bytes()[..16]);
            with_net = unlocker.with_net;
        }

        thread::Builder::new().name("webui".into()).spawn(move || {
            info!("start webui");
            Self::run_webui(receiver, with_net, peer_representation).or_else(|forward| {
                error!("error from webui-server: {}", forward);
                Err(forward)
            })?;
            info!("stopped webui");
            Ok::<(), std::io::Error>(())
        })
    }

    fn spawn_tui(
        this: Arc<Mutex<Self>>,
        resender: Sender<InternalUiMsg>,
        receiver: Receiver<InternalUiMsg>,
    ) -> Result<thread::JoinHandle<Result<(), std::io::Error>>, std::io::Error> {
        let title;
        let paths;
        let with_net;
        // lock block
        {
            let unlock = this.lock().unwrap();
            title = unlock.peer_id.to_string().clone();
            paths = unlock.paths.clone();
            with_net = unlock.with_net.clone();
        }

        std::thread::Builder::new()
            .name("tui".into())
            .spawn(move || {
                info!("starting tui");

                // do finally the necessary
                // this blocks this async future
                Self::run_tui(title, paths, with_net, receiver, resender).map_err(
                    |error_text| std::io::Error::new(std::io::ErrorKind::Other, error_text),
                )?;
                info!("stopped tui");
                Ok::<(), std::io::Error>(())
            })
    }

    fn spawn_message_loop(
        receiver: Receiver<UiUpdateMsg>,
        multiplex_send: Vec<Sender<InternalUiMsg>>,
    ) -> Result<thread::JoinHandle<()>, std::io::Error> {
        thread::Builder::new()
            .name("ui msg".into())
            .spawn(move || loop {
                if !Self::run_message_forwarding(&receiver, &multiplex_send) {
                    break;
                }
            })
    }

    /// Run the UIs - there is less controlling rather than showing
    fn run_tui(
        title: String,
        paths: Vec<String>,
        with_net: bool,
        tui_receiver: Receiver<InternalUiMsg>,
        resender: Sender<InternalUiMsg>,
    ) -> Result<(), String> {
        info!("tui about to run");

        // set up communication for tui messages
        info!("spawning tui async thread");
        let mut tui = Tui::new(title, &paths, with_net)?;

        task::block_on(async move {
            // message and refresh tui loop
            loop {
                // due to pressing 'q' tui will stop and hence also the loop
                if !tui.refresh().await {
                    break;
                }
                tui.run_cursive(&resender, &tui_receiver).await;
            }
        });
        Ok(())
    }

    /// Run the controller
    fn run_webui(
        webui_receiver: Receiver<InternalUiMsg>,
        net_support: bool,
        peer_representation: PeerRepresentation,
    ) -> io::Result<()> {
        if webbrowser::open(&["http://", config::net::WEBSOCKET_ADDR].concat()).is_err() {
            info!("Could not open browser!");
        }

        task::block_on(async move {
            // loop non blocking

            info!("spawning webui async thread");
            let webui = WebUI::new(peer_representation, net_support);
            webui.run(webui_receiver).await
        })
    }

    /// This basically wraps incoming UiUpdateMsg to InternalUiMsg
    /// which kind of defines an extra layer for convenience, and to
    /// be extended and so on.
    fn run_message_forwarding(
        receiver: &Receiver<UiUpdateMsg>,
        multiplex_send: &Vec<Sender<InternalUiMsg>>,
    ) -> bool {
        if let Ok(forward_sys_message) = receiver.try_recv() {
            match forward_sys_message {
                UiUpdateMsg::NetUpdate((recv_dialog, text)) => {
                    trace!("net update forwarding");
                    // todo: create a closure/fn to do a multiple send
                    let outter_containment = (recv_dialog, text);
                    for forward_sender in multiplex_send {
                        forward_sender
                            .send(InternalUiMsg::Update(outter_containment.clone()))
                            .unwrap();
                    }
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
                    for forward_sender in multiplex_send {
                        forward_sender
                            .send(InternalUiMsg::StartAnimate(signal.clone(), on_off.clone()))
                            .unwrap();
                    }
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
