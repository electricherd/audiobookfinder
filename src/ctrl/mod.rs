//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.

pub mod tui; // todo: pub is not recommended, I use it for doctest
mod webui;

use self::tui::Tui;
use self::webui::WebUI;
use super::common::startup::{StartUp, SyncStartUp};
use super::config;

use async_std::task;
use crossbeam::sync::WaitGroup;
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
#[derive(Clone, Serialize, Deserialize)]
pub enum CollectionPathAlive {
    BusyPath(usize),
    HostSearch,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone)]
pub struct ForwardNetMessage {
    net: NetMessages,
    cnt: String,
}

impl ForwardNetMessage {
    pub fn new(net: NetMessages, cnt: String) -> Self {
        Self { net, cnt }
    }
}

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

pub struct Ctrl {
    peer_id: PeerId,
    paths: Vec<String>,
    with_net: bool,
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
    pub fn new(new_id: PeerId, paths: &Vec<String>, with_net: bool) -> Self {
        Self {
            peer_id: new_id,
            paths: paths.clone(),
            with_net,
        }
    }

    pub fn run(
        new_id: PeerId,
        paths: &Vec<String>,
        receiver: Receiver<UiUpdateMsg>,
        with_net: bool,
        wait_main: WaitGroup,
        has_webui: bool,
        has_tui: bool,
    ) -> Result<(), std::io::Error> {
        // sync both sub uis
        let ui_waitgroup = WaitGroup::new();

        // create instance which will be passed into the different uis
        let instance = Ctrl::new(new_id, paths, with_net);

        let arc_self_tui = Arc::new(Mutex::new(instance));
        let arc_self_webui = arc_self_tui.clone();

        // all senders that UiUpdateMessages will be forwarded to
        let mut internal_senders: Vec<Sender<InternalUiMsg>> = vec![];

        // 1) tui thread
        let thread_tui = if has_tui {
            let tui_waitgroup = ui_waitgroup.clone();
            let (sender_to_register, receiver_to_tui_thread) = channel::<InternalUiMsg>();
            let resender = sender_to_register.clone();
            internal_senders.push(sender_to_register);
            Self::spawn_tui(
                arc_self_tui,
                resender,
                receiver_to_tui_thread,
                tui_waitgroup,
            )?
        } else {
            std::thread::spawn(|| Ok(()))
        };

        // 2) webui thread
        let thread_webui = if has_webui {
            let wui_waitgroup = ui_waitgroup.clone();
            let (sender_to_register, receiver_to_web_ui_thread) = channel::<InternalUiMsg>();
            internal_senders.push(sender_to_register);
            Self::spawn_webui(arc_self_webui, receiver_to_web_ui_thread, wui_waitgroup)?
        } else {
            // empty thread
            std::thread::spawn(|| Ok(()))
        };

        // 3) ui message forwarding loop thread
        let message_loop = Self::spawn_message_loop(receiver, internal_senders);

        // A) wait for sub syncs in order ...
        info!("syncing with 2 other sub threads webui and tui");
        ui_waitgroup.wait();
        info!("synced with 2 other sub threads webui and tui");
        // B) ... to unlock sync/block startup with main thread
        // we are ready: up and listening!!
        info!("waiting for main thread sync");
        wait_main.wait();
        info!("synced with main thread");

        // todo: if tui and webui both run, both have to be joined AT THE SAME time in order
        //       to know when they are ended
        // todo: NOW only with tui (not webui) the "KEEP" option from main thread is working
        let res_tui = thread_tui.join();
        // todo: there is no quitting yet ... is there???
        drop(message_loop);
        drop(thread_webui);
        Ok::<(), std::io::Error>(())
    }

    fn spawn_webui(
        this: Arc<Mutex<Self>>,
        receiver: Receiver<InternalUiMsg>,
        wait_ui_sync: WaitGroup,
    ) -> Result<thread::JoinHandle<Result<(), std::io::Error>>, std::io::Error> {
        let mut peer_representation: PeerRepresentation = [0 as u8; 16];
        let with_net;
        let paths;
        // lock block
        {
            let unlocker = this.lock().unwrap();
            paths = unlocker.paths.clone();
            peer_representation.copy_from_slice(&unlocker.peer_id.as_bytes()[..16]);
            with_net = unlocker.with_net;
        }

        thread::Builder::new().name("webui".into()).spawn(move || {
            info!("start webui");
            Self::run_webui(receiver, with_net, peer_representation, paths, wait_ui_sync).or_else(
                |forward| {
                    error!("error from webui-server: {}", forward);
                    Err(forward)
                },
            )?;
            info!("stopped webui");
            Ok::<(), std::io::Error>(())
        })
    }

    fn spawn_tui(
        this: Arc<Mutex<Self>>,
        resender: Sender<InternalUiMsg>,
        receiver: Receiver<InternalUiMsg>,
        sync_startup: WaitGroup,
    ) -> Result<thread::JoinHandle<Result<(), std::io::Error>>, std::io::Error> {
        let title;
        let paths;
        let with_net;
        // lock block
        {
            let unlocker = this.lock().unwrap();
            title = unlocker.peer_id.to_string().clone();
            paths = unlocker.paths.clone();
            with_net = unlocker.with_net.clone();
        }

        std::thread::Builder::new()
            .name("tui".into())
            .spawn(move || {
                trace!("tui waits for sync");
                // synchronizing
                sync_startup.wait();
                trace!("tui starts");
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
        paths: Vec<String>,
        wait_ui_sync: WaitGroup,
    ) -> io::Result<()> {
        if webbrowser::open(&["http://", config::net::WEBSOCKET_ADDR].concat()).is_err() {
            info!("Could not open browser!");
        }

        task::block_on(async move {
            info!("spawning webui async thread");
            let webui = WebUI::new(peer_representation, net_support, paths);
            webui.run(webui_receiver, wait_ui_sync).await
        })
    }

    /// This basically wraps incoming UiUpdateMsg to InternalUiMsg
    /// which kind of defines an extra layer for convenience, and to
    /// be extended and so on.
    fn run_message_forwarding(
        receiver: &Receiver<UiUpdateMsg>,
        multiplex_send: &Vec<Sender<InternalUiMsg>>,
    ) -> bool {
        if let Ok(forward_sys_message) = receiver.recv() {
            match forward_sys_message {
                UiUpdateMsg::NetUpdate(ForwardNetMessage {
                    net: recv_dialog,
                    cnt: text,
                }) => {
                    trace!("net update forwarding");
                    // todo: create a closure/fn to do a multiple send
                    let outter_containment = ForwardNetMessage::new(recv_dialog, text);
                    for forward_sender in multiplex_send {
                        forward_sender
                            .send(InternalUiMsg::Update(outter_containment.clone()))
                            .unwrap_or_else(|_| {
                                warn!("forwarding message cancelled probably due to quitting!");
                            });
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
                            .unwrap_or_else(|_| {
                                warn!("forwarding message cancelled probably due to quitting!");
                            });
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
}
