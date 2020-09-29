//! The forwarder module shall act as wrapper for the ffi mangled lib.rs
//! calls, which shall be very thin and spare.
use crate::{
    common::paths::SearchPath,
    ctrl::{ForwardNetMsg, UiUpdateMsg},
    data::{collection::Collection, ipc::IPC},
    net::subs::peer_representation,
    shared,
};
use crossbeam::{sync::WaitGroup, unbounded};
use std::sync::{mpsc::channel, Arc, Mutex};

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
    let wait_net_thread = WaitGroup::new();
    let wait_net_thread_trigger = wait_net_thread.clone();
    //
    let (ui_sender, reactor) = channel::<UiUpdateMsg>();

    // nothing to send to net yet
    let (_, dummy_ipc_receive) = unbounded::<IPC>();

    let _what = shared::net_search(wait_net_thread, Some(ui_sender), dummy_ipc_receive);

    let mut out = 0;

    // try to trigger here if even necessary (single one doesn't wait at all??)
    wait_net_thread_trigger.wait();

    // loop over (cheap version here, but we are in async, so
    while let Some(reaction) = reactor.try_iter().next() {
        match reaction {
            UiUpdateMsg::CollectionUpdate(_, _) => {}
            UiUpdateMsg::NetUpdate(net_message) => {
                match net_message {
                    ForwardNetMsg::Add(peer) => {
                        // yet only id is interesting
                        out = peer_representation::peer_to_hash(&peer.id);
                        // break;
                    }
                    ForwardNetMsg::Delete(_peer_id) => {}
                    ForwardNetMsg::Stats(_) => {}
                }
            }
            UiUpdateMsg::PeerSearchFinished(_peer_id, _data) => {
                //
            }
            UiUpdateMsg::StopUI => {}
        }
    }
    out
}
