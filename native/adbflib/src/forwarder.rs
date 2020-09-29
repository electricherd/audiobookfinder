//! The forwarder module shall act as wrapper for the ffi mangled lib.rs
//! calls, which shall be very thin and spare.
use crate::{
    common::paths::SearchPath,
    ctrl::UiUpdateMsg,
    data::{collection::Collection, ipc::IPC},
    shared,
};
use crossbeam::{sync::WaitGroup, unbounded};
use std::sync::{mpsc::channel, Arc, Mutex};

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

/// return the peer hash for testing
pub async fn ffi_new_peer() -> u64 {
    // ???? for ui messages??
    let has_ui = true;
    //
    let wait_net_thread = WaitGroup::new();
    //
    let (ui_sender, reactor) = channel::<UiUpdateMsg>();
    let (_, ipc_receive) = unbounded::<IPC>();

    let _what = shared::net_search(has_ui, wait_net_thread, ui_sender, ipc_receive);

    //match reactor.
    0
}
