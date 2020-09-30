//! The forwarder module shall act as wrapper for the ffi mangled lib.rs
//! calls, which shall be very thin and spare there, but extensive here
//! to prepare clean calls to adbfflutter.
use crate::{
    common::paths::SearchPath,
    ctrl::{ForwardNetMsg, UiUpdateMsg},
    data::{collection::Collection, ipc::IPC},
    net::subs::peer_representation,
    shared,
};
use async_std::task;
use crossbeam::{sync::WaitGroup, unbounded};
use std::{
    sync::{mpsc::channel, Arc, Mutex},
    thread,
};

/// just return the number of audio files found for now
pub fn ffi_file_count_good(input_path: Vec<String>) -> u32 {
    // prepare data
    let cleaned_paths = SearchPath::new(&input_path);
    let search_path = Arc::new(Mutex::new(cleaned_paths));

    let (tx, _rx) = channel::<UiUpdateMsg>();
    let synced_to_ui_messages = Arc::new(Mutex::new(tx.clone()));
    let has_ui = false;

    // set up data
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

/// return the peer hash for testing yet
pub async fn ffi_new_peer() -> u64 {
    //
    let dummy_wait_net_thread = WaitGroup::new();
    let (ui_sender, reactor) = channel::<UiUpdateMsg>();
    let (_, dummy_ipc_receive) = unbounded::<IPC>();

    // initiate a thread (which will be ... (todo: lazy_static maybe later, and keep it running)
    // which handles net communication
    let single_shot_net_thread = thread::Builder::new()
        .name("app_net".into())
        .spawn(move || {
            task::block_on(async move {
                let _ =
                    shared::net_search(dummy_wait_net_thread, Some(ui_sender), dummy_ipc_receive)
                        .await;
            });
        })
        .unwrap();

    // very interesting, the compiler is awesome!!
    let out;

    // loop over fixme: (very cheap version here, that could be done more elegantly)
    loop {
        if let Ok(reaction) = reactor.try_recv() {
            match reaction {
                UiUpdateMsg::CollectionUpdate(_, _) => {}
                UiUpdateMsg::NetUpdate(net_message) => {
                    match net_message {
                        ForwardNetMsg::Add(peer) => {
                            // yet only id is interesting
                            out = peer_representation::peer_to_hash(&peer.id);
                            break;
                        }
                        ForwardNetMsg::Delete(_peer_id) => {}
                        ForwardNetMsg::Stats(_) => {}
                    }
                }
                UiUpdateMsg::PeerSearchFinished(_peer_id, _data) => {}
                UiUpdateMsg::StopUI => {}
            }
        }
    }
    // kill this big thread / fixme: very inefficent yet
    drop(single_shot_net_thread);
    out
}
