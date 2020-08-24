//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.
pub mod tui; // todo: pub is not recommended, I use it for doctest
mod webui;

use self::{tui::Tui, webui::WebUI};
use super::{
    config,
    net::peer_representation::{self, PeerRepresentation},
    paths::SearchPath,
};
use async_std::task;
use crossbeam::sync::WaitGroup;
use libp2p_core::PeerId;
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
    Debug(String),
    ShowStats { show: NetStats },
}

#[derive(Clone)]
pub struct UiPeer {
    //
    pub id: PeerId,
    pub addresses: Vec<String>,
}

#[derive(Clone)]
pub enum ForwardNetMessage {
    Add(UiPeer),
    Delete(PeerId),
    Stats(NetMessages),
}

/// internal messages inside ui
pub enum InternalUiMsg {
    Update(ForwardNetMessage),
    StartAnimate(CollectionPathAlive, Status),
    StepAndAnimate(CollectionPathAlive),
    PeerSearchFinished(PeerId, u32),
    Terminate,
}

#[derive(Clone)]
pub enum UiUpdateMsg {
    NetUpdate(ForwardNetMessage),
    CollectionUpdate(CollectionPathAlive, Status),
    PeerSearchFinished(PeerId, u32),
    StopUI,
}

#[derive(Copy, Clone)]
pub struct NetStats {
    pub line: usize,
    pub max: usize,
}

enum Finisher {
    TUI,
    WEBUI,
}

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
    /// * 'receiver' - The receiver that takes incoming ctrl messages
    /// * 'with_net' - If ctrl should consider net messages
    /// * 'sync_sender' - For start up that main can be informed, "I'm ready"
    pub fn new(new_id: PeerId, paths: Arc<Mutex<SearchPath>>, with_net: bool) -> Self {
        Self {
            peer_id: new_id,
            paths: paths.clone(),
            with_net,
        }
    }

    pub fn run(
        new_id: PeerId,
        paths: Arc<Mutex<SearchPath>>,
        receiver: Receiver<UiUpdateMsg>,
        with_net: bool,
        wait_main: WaitGroup,
        has_webui: bool,
        has_tui: bool,
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
        let (sender_tui_to_register, receiver_to_tui_thread) = channel::<InternalUiMsg>();
        let sender_tui_only_to_finish = sender_tui_to_register.clone();
        let thread_tui = if has_tui {
            let resender = sender_tui_to_register.clone();
            let tui_waitgroup = wait_all_uis.clone();
            let thread_finisher_tui = thread_finisher.clone();

            internal_senders.push(sender_tui_to_register);
            Self::spawn_tui(
                arc_self_tui,
                resender,
                receiver_to_tui_thread,
                tui_waitgroup,
                thread_finisher_tui,
            )?
        } else {
            std::thread::spawn(|| Ok(()))
        };

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
                Finisher::TUI => {
                    info!("TUI finished first, so send to terminate WEBUI!");
                    sender_wui.send(InternalUiMsg::Terminate).unwrap();
                    let to_pass_through = thread_tui.join().unwrap();
                    drop(forwarding_message_loop); // let drop message loop only after joining!!!
                    to_pass_through
                }
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
                drop(thread_tui);
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
            let peer_bytes = &unlocker.peer_id.as_ref();
            hasher.write(peer_bytes);
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

    fn spawn_tui(
        this: Arc<Mutex<Self>>,
        resender: Sender<InternalUiMsg>,
        receiver: Receiver<InternalUiMsg>,
        sync_startup: WaitGroup,
        thread_finisher: Sender<Finisher>,
    ) -> Result<thread::JoinHandle<Result<(), std::io::Error>>, std::io::Error> {
        let title;
        let paths;
        let with_net;
        // lock block
        {
            let unlocker = this.lock().unwrap();
            title = peer_representation::peer_to_hash_string(&unlocker.peer_id);
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
                let fix_path = paths.lock().unwrap().read();
                Self::run_tui(title, fix_path, with_net, receiver, resender).map_err(
                    |error_text| std::io::Error::new(std::io::ErrorKind::Other, error_text),
                )?;
                info!("stopped tui");

                // send finisher since it should also stop webui
                thread_finisher.send(Finisher::TUI).unwrap_or_else(|_| {
                    info!("probably receiver got webui finisher first!");
                });

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
        paths: Arc<Mutex<SearchPath>>,
        wait_ui_sync: WaitGroup,
        open_browser: bool,
        web_port: u16,
    ) -> io::Result<()> {
        let mut wait_ui_sync_maybe_before = None;
        if open_browser {
            if webbrowser::open(
                // todo: what if https
                &["http://", config::net::WEB_ADDR, ":", &web_port.to_string()].concat(),
            )
            .is_ok()
            {
                // it's clear this browser will take websocket start
                wait_ui_sync_maybe_before = Some(wait_ui_sync);
            } else {
                // todo: have to do something
                error!("Could not open browser!");
            }
        } else {
            // important:
            // if no open browser the other threads don't depend on
            // sync with this, so release the wait for sync with other threads and
            // continue.
            wait_ui_sync.wait();
            wait_ui_sync_maybe_before = None;
        }

        task::block_on(async move {
            info!("spawning webui async thread");
            let webui = WebUI::new(peer_representation, net_support, paths);
            webui
                .run(webui_receiver, wait_ui_sync_maybe_before, web_port)
                .await
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
                UiUpdateMsg::NetUpdate(forward_net_message) => {
                    match forward_net_message {
                        ForwardNetMessage::Stats(_net_message) => {
                            // todo: implement stats here
                        }
                        ForwardNetMessage::Add(peer_to_add) => {
                            for forward_sender in multiplex_send {
                                forward_sender
                                    .send(InternalUiMsg::Update( ForwardNetMessage::Add( peer_to_add.clone())))
                                    .unwrap_or_else(|_| {
                                        warn!("forwarding message cancelled probably due to quitting!");
                                    });
                            }
                        }
                        ForwardNetMessage::Delete(peer_id_to_remove) => {
                            for forward_sender in multiplex_send {
                                forward_sender
                                    .send(InternalUiMsg::Update( ForwardNetMessage::Delete( peer_id_to_remove.clone())))
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
                UiUpdateMsg::PeerSearchFinished(peer_representation, count) => {
                    for forward_sender in multiplex_send {
                        forward_sender
                            .send(InternalUiMsg::PeerSearchFinished(
                                peer_representation.clone(),
                                count.clone(),
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
