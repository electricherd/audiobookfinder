//! The forwarder module shall act as wrapper for the ffi mangled lib.rs
//! calls, which shall be very thin and spare there, but extensive here
//! to prepare clean calls to adbfflutter.
use crate::{
    common::paths::SearchPath,
    ctrl::{ForwardNetMsg, UiUpdateMsg},
    data::{
        collection::Collection,
        ipc::{
            IFCollectionOutputData,
            IPC::{self, DoneSearching},
        },
    },
    net::subs::peer_representation::peer_to_hash_string,
    shared,
};
use async_std::task;
use crossbeam::{
    channel::{unbounded, Receiver, Sender},
    sync::WaitGroup,
};
use libp2p::core::PeerId;
use serde_json;
use std::{
    sync::{Arc, Mutex},
    thread,
};

lazy_static! {
    /// a static immutable runtime for all network activity
    static ref NET_RUNTIME: Mutex<(Receiver<UiUpdateMsg>,Sender<IPC>)> = Mutex::new(create_net_runtime());
    /// a static mutable data collection
    static ref NET_UI : Mutex<UIList> = Mutex::new(UIList { cnt: Vec::new() });
}

/// just return the number of audio files found for now
pub fn ffi_file_count_good(input_path: Vec<String>) -> u32 {
    // prepare data
    let cleaned_paths = SearchPath::new(&input_path);
    let search_path = Arc::new(Mutex::new(cleaned_paths));

    let (tx, _rx) = unbounded::<UiUpdateMsg>();
    let synced_to_ui_messages = Arc::new(Mutex::new(tx.clone()));
    let has_ui = false;

    // set up data and run search
    let collection_protected = Arc::new(Mutex::new(Collection::new()));
    let output_data_return_handle = shared::collection_search(
        collection_protected,
        search_path,
        synced_to_ui_messages,
        has_ui,
    );

    // scope and block trickery for lifetime and mutability
    output_data_return_handle.nr_found_songs
}

/// return the json of peer uis
pub async fn ffi_ui_messages_as_json() -> String {
    // todo: unwrap using NET_UI is save unless you
    //       run this function simultaneously many instances, secure this because
    //       at least 2 functions now use net component, and simultaneous usage
    //       is more probable now!

    // get network runtime
    let (net_receiver, _) = &*NET_RUNTIME.lock().unwrap();

    // break loop if a yet noticeable change happens
    loop {
        if let Ok(reaction) = net_receiver.recv() {
            match reaction {
                UiUpdateMsg::CollectionUpdate(_, _) => {}
                UiUpdateMsg::NetUpdate(net_message) => match net_message {
                    ForwardNetMsg::Add(peer) => {
                        let ui_list = &mut NET_UI.lock().unwrap();
                        {
                            ui_list.add_peer(&peer.id);
                            break;
                        }
                    }
                    ForwardNetMsg::Delete(peer_id) => {
                        let ui_list = &mut NET_UI.lock().unwrap();
                        {
                            ui_list.remove_peer(&peer_id);
                            break;
                        }
                    }
                    ForwardNetMsg::Stats(_) => {}
                },
                UiUpdateMsg::PeerSearchFinished(peer_id, data) => {
                    let ui_list = &mut NET_UI.lock().unwrap();
                    {
                        ui_list.add_search_finished(&peer_id, &data);
                        break;
                    }
                }
                UiUpdateMsg::StopUI => unreachable!(),
            }
        }
    }
    let ui_list = &NET_UI.lock().unwrap();
    {
        let json: &UIList = &*ui_list;
        let peers_ui_json = serde_json::to_string(&json.cnt).unwrap();
        peers_ui_json
    }
}

/// send ipc found out
pub async fn ffi_send_ipc_search_done(nr_searched_files: u32, nr_found_songs: u32) -> bool {
    //
    // get network runtime
    let (_, ipc_sender) = &mut *NET_RUNTIME.lock().unwrap();
    let sending = IFCollectionOutputData {
        nr_searched_files,
        nr_found_songs,
        size_of_data_in_kb: 0,
        nr_internal_duplicates: 0,
    };
    ipc_sender.send(DoneSearching(sending)).is_ok()
}

// ------------------------------------------------------------------------------------------

/// Opens a net thread and return ui message receiver and ipc message sender
/// to be used as static instance for Dart-to-backend communication.
fn create_net_runtime() -> (Receiver<UiUpdateMsg>, Sender<IPC>) {
    // outgoing crossbeam receiver
    let (ui_sender, reactor) = unbounded::<UiUpdateMsg>();
    let (ipc_sender, ipc_receive) = unbounded::<IPC>();
    thread::Builder::new()
        .name("app_net".into())
        .spawn(move || {
            task::block_on(async move {
                // mock input parameters
                let dummy_wait_net_thread = WaitGroup::new();

                match shared::net_search(dummy_wait_net_thread, Some(ui_sender), ipc_receive).await
                {
                    Ok(_) => {}
                    Err(e) => error!("network error: {}", e),
                }
            });
        })
        .unwrap();
    (reactor, ipc_sender)
}

/// Struct viewable for ffi inner part and used as json export
/// !!! change Dart names as well, cause variable names MUST be in sync
#[derive(Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
struct UIListInner {
    // normally peer would be the hash key, but since we use it as json container
    // it's a bit itchy
    peerid: String,
    finished: i32,
    searched: u32,
}
/// Container and helper to fill UIListInner easier
struct UIList {
    pub cnt: Vec<UIListInner>,
}
impl UIList {
    fn add_peer(&mut self, peer_id: &PeerId) {
        let peer_string = peer_to_hash_string(peer_id);
        if self.cnt.iter().find(|e| e.peerid == peer_string).is_none() {
            self.cnt.push(UIListInner {
                peerid: peer_string,
                finished: -1, // -1 on dart side should show that it is not yet completed
                searched: 0,
            });
        }
    }
    fn remove_peer(&mut self, peer_id: &PeerId) {
        //todo: as soon as this is not experimental:
        //      self.cnt.drain_filter(|e| e.peer_id == peer_to_hash(peer_id));
        let peer_string = peer_to_hash_string(peer_id);
        let mut i = 0;
        while i != self.cnt.len() {
            if self.cnt[i].peerid == peer_string {
                self.cnt.remove(i);
                break;
            } else {
                i += 1;
            }
        }
    }
    fn add_search_finished(&mut self, peer_id: &PeerId, data: &IFCollectionOutputData) {
        let peer_string = peer_to_hash_string(peer_id);
        let mut i = 0;
        while i != self.cnt.len() {
            if self.cnt[i].peerid == peer_string {
                self.cnt[i].finished = data.nr_found_songs as i32;
                self.cnt[i].searched = data.nr_searched_files;
                break;
            } else {
                i += 1;
            }
        }
    }
}
