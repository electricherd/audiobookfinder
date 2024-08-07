//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.
mod webui;

use self::webui::WebUI;
use super::{
    common::{config, paths::SearchPath},
    data::ipc::IFCollectionOutputData,
    net::subs::peer_representation::PeerRepresentation,
};
use async_std::task;
use crossbeam::{channel::Receiver as CReceiver, sync::WaitGroup};
use libp2p::core::PeerId;
use std::{
    collections::hash_map::DefaultHasher,
    hash::Hasher,
    io,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

/// alive Signal for path from collector or net search alive
#[derive(Clone, Serialize, Deserialize)]
pub enum CollectionPathAlive {
    BusyPath(usize),
    HostSearch,
}
/// Turn on/off something
#[derive(Clone, Serialize)]
pub enum Status {
    ON,
    OFF,
}
/// Yet unimportant net messages todo: make it better!
#[derive(Clone)]
pub enum NetInfoMsg {
    Debug(String),
    ShowStats { show: NetStatsMsg },
}
/// Peer string identificators
#[derive(Clone)]
pub struct UiClientPeer {
    //
    pub id: PeerId,
    pub addresses: Vec<String>,
}
/// Forwarding net messages
#[derive(Clone)]
pub enum ForwardNetMsg {
    Add(UiClientPeer),
    Delete(PeerId),
    Stats(NetInfoMsg),
}

/// Internal messages inside UI
pub enum InternalUiMsg {
    Update(ForwardNetMsg),
    StartAnimate(CollectionPathAlive, Status),
    StepAndAnimate(CollectionPathAlive),
    PeerSearchFinished(PeerId, IFCollectionOutputData),
    Terminate,
}

/// UI updating messages
#[derive(Clone)]
pub enum UiUpdateMsg {
    NetUpdate(ForwardNetMsg),
    CollectionUpdate(CollectionPathAlive, Status),
    PeerSearchFinished(PeerId, IFCollectionOutputData),
    StopUI,
}

/// NetStats message
#[derive(Copy, Clone)]
pub struct NetStatsMsg {
    pub line: usize,
    pub max: usize,
}

enum Finisher {
    WEBUI,
}

/// The controller holds user interfaces as webui, tui. It currently creates
/// and runs the user interfaces, distributes messages and sends out messages
/// to be used somewhere else.
pub struct Ctrl {
    peer_id: PeerId,
    paths: Arc<Mutex<SearchPath>>,
    with_net: bool,
}

impl Ctrl {
    /// Create a new controller if everything fits.
    ///
    /// # Arguments
    /// * 'peer_id' - The peer_id this client/server uses
    /// * 'paths' - The paths that will be searched
    /// * 'with_net' - If ctrl should consider net messages
    fn new(new_id: PeerId, paths: Arc<Mutex<SearchPath>>, with_net: bool) -> Self {
        Self {
            peer_id: new_id,
            paths: paths.clone(),
            with_net,
        }
    }
    /// Create a new controller if everything fits.
    ///
    /// # Arguments
    /// * 'new_id' - The peer_id this client/server uses
    /// * 'paths' - The paths that will be searched
    /// * 'receiver' - The paths that will be searched
    /// * 'with_net' - If ctrl should consider net messages
    /// * 'wait_main' - The main thread notifier
    /// * 'has_webui' - If webui has to be considered
    /// * 'has_tui' - If tui has to be considered
    /// * 'open_browser' - If browser should be automatically opened
    /// * 'web_port' - Browser, webui port to use
    pub fn run(
        new_id: PeerId,
        paths: Arc<Mutex<SearchPath>>,
        receiver: CReceiver<UiUpdateMsg>,
        with_net: bool,
        wait_main: WaitGroup,
        has_webui: bool,
        open_browser: bool,
        web_port: u16,
    ) -> Result<(), std::io::Error> {
        // sync both sub uis
        let wait_all_uis = WaitGroup::new();

        let (thread_finisher, finish_threads) = channel::<Finisher>();

        // create instance which will be passed into the different uis
        let instance = Ctrl::new(new_id, paths, with_net);

        let arc_self_tui = Arc::new(Mutex::new(instance));
        let arc_self_webui = arc_self_tui.clone();

        // all senders that UiUpdateMessages will be forwarded to
        let mut internal_senders: Vec<Sender<InternalUiMsg>> = vec![];

        // 1) tui thread
        let (sender_tui_only_to_finish, _receiver_to_tui_thread) = channel::<InternalUiMsg>();

        // 2) webui thread
        let (sender_wui, receiver_to_web_ui_thread) = channel::<InternalUiMsg>();
        let thread_webui = if has_webui {
            let sender_to_register = sender_wui.clone();
            let wui_waitgroup = wait_all_uis.clone();
            let thread_finisher_tui = thread_finisher.clone();

            internal_senders.push(sender_to_register);
            Self::spawn_webui(
                arc_self_webui,
                receiver_to_web_ui_thread,
                wui_waitgroup,
                thread_finisher_tui,
                open_browser,
                web_port,
            )?
        } else {
            // empty thread
            std::thread::spawn(|| Ok(()))
        };

        // 3) ui message forwarding loop thread
        let forwarding_message_loop = Self::spawn_message_loop(receiver, internal_senders);

        // A) wait for sub syncs in order ...
        info!("syncing with 2 other sub threads webui and tui");
        wait_all_uis.wait();
        info!("synced with 2 other sub threads webui and tui");
        // B) ... to unlock sync/block startup with main thread
        // we are ready: up and listening!!
        info!("waiting for main thread sync");
        wait_main.wait();
        info!("synced with main thread");

        // either of these can finish and we want to block!
        match finish_threads.recv() {
            Ok(finished) => match finished {
                Finisher::WEBUI => {
                    info!("WEBUI finished first, so send to terminate TUI!");
                    sender_tui_only_to_finish
                        .send(InternalUiMsg::Terminate)
                        .unwrap();
                    let to_pass_through = thread_webui.join().unwrap();
                    drop(forwarding_message_loop); // let drop forwarding message loop only after joining!!!!
                    to_pass_through
                }
            },
            Err(e) => {
                error!("something really bad happenend: {}!!", e);
                drop(thread_webui);
                drop(forwarding_message_loop);
                // todo: make a new error
                Ok::<(), std::io::Error>(())
            }
        }
    }

    fn spawn_webui(
        this: Arc<Mutex<Self>>,
        receiver: Receiver<InternalUiMsg>,
        wait_ui_sync: WaitGroup,
        thread_finisher: Sender<Finisher>,
        open_browser: bool,
        web_port: u16,
    ) -> Result<thread::JoinHandle<Result<(), std::io::Error>>, std::io::Error> {
        let with_net;
        let paths;
        // lock block
        let mut hasher = DefaultHasher::new();
        {
            let unlocker = this.lock().unwrap();
            paths = unlocker.paths.clone();
            with_net = unlocker.with_net;
            let peer_bytes = unlocker.peer_id.to_bytes();
            hasher.write(peer_bytes.as_ref());
        }
        let peer_representation = hasher.finish();

        thread::Builder::new().name("webui".into()).spawn(move || {
            info!("start webui");
            Self::run_webui(
                receiver,
                with_net,
                peer_representation,
                paths,
                wait_ui_sync,
                open_browser,
                web_port,
            )
            .or_else(|forward| {
                error!("error from webui-server: {}", forward);
                Err(forward)
            })?;
            info!("stopped webui");

            // send finish
            thread_finisher.send(Finisher::WEBUI).unwrap_or_else(|_| {
                info!("probably receiver got tui finisher first!");
            });

            Ok::<(), std::io::Error>(())
        })
    }

    fn spawn_message_loop(
        receiver: CReceiver<UiUpdateMsg>,
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

    /// Run the controller
    fn run_webui(
        webui_receiver: Receiver<InternalUiMsg>,
        net_support: bool,
        peer_representation: PeerRepresentation,
        paths: Arc<Mutex<SearchPath>>,
        wait_ui_sync: WaitGroup,
        open_browser: bool,
        web_port: u16,
    ) -> io::Result<()> {
        if open_browser {
            // fixme: fix this to not be included in library in some point
            if !try_open_browser(web_port) {
                error!("Could not open browser!");
                println!(
                    "Could not open browser, try opening manually: http://{}:{} to start!",
                    config::net::WEB_ADDR,
                    web_port
                );
            }
        }

        task::block_on(async move {
            info!("spawning webui async thread");
            let webui = WebUI::new(peer_representation, net_support, paths);
            webui.run(webui_receiver, wait_ui_sync, web_port).await
        })
    }

    /// This basically wraps incoming UiUpdateMsg to InternalUiMsg
    /// which kind of defines an extra layer for convenience, and to
    /// be extended and so on.
    fn run_message_forwarding(
        receiver: &CReceiver<UiUpdateMsg>,
        multiplex_send: &Vec<Sender<InternalUiMsg>>,
    ) -> bool {
        if let Ok(forward_sys_message) = receiver.recv() {
            match forward_sys_message {
                UiUpdateMsg::NetUpdate(forward_net_message) => {
                    match forward_net_message {
                        ForwardNetMsg::Stats(_net_message) => {
                            // todo: implement stats here
                        }
                        ForwardNetMsg::Add(peer_to_add) => {
                            for forward_sender in multiplex_send {
                                forward_sender
                                    .send(InternalUiMsg::Update( ForwardNetMsg::Add( peer_to_add.clone())))
                                    .unwrap_or_else(|_| {
                                        warn!("forwarding message cancelled probably due to quitting!");
                                    });
                            }
                        }
                        ForwardNetMsg::Delete(peer_id_to_remove) => {
                            for forward_sender in multiplex_send {
                                forward_sender
                                    .send(InternalUiMsg::Update( ForwardNetMsg::Delete( peer_id_to_remove.clone())))
                                    .unwrap_or_else(|_| {
                                        warn!("forwarding message cancelled probably due to quitting!");
                                    });
                            }
                        }
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
                UiUpdateMsg::PeerSearchFinished(peer_representation, data) => {
                    for forward_sender in multiplex_send {
                        forward_sender
                            .send(InternalUiMsg::PeerSearchFinished(
                                peer_representation.clone(),
                                data.clone(),
                            ))
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

// see https://doc.rust-lang.org/reference/conditional-compilation.html
// same configuration as "open" from "webbrowser" crate allows
#[cfg(any(
    target_os = "android",
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "freebsd",
    targest_os = "netbsd",
    target_os = "openbsd",
    target_os = "haiku",
    target_arch = "wasm32"
))]
fn try_open_browser(web_port: u16) -> bool {
    webbrowser::open(
        // todo: what if https
        &["http://", config::net::WEB_ADDR, ":", &web_port.to_string()].concat(),
    )
    .is_ok()
}

#[cfg(not(any(
    target_os = "android",
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "haiku",
    target_arch = "wasm32"
)))]
pub fn try_open_browser(web_port: u16) -> bool {
    false
}
