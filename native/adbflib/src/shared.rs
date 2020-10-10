//! The shared module shall give binary and library high level functionality
//! to be used by both the same way.
use crate::{
    common::paths::SearchPath,
    ctrl::UiUpdateMsg,
    data::{
        self, audio_info::Container, collection::Collection, ipc::IPC,
        IFInternalCollectionOutputData,
    },
    net::Net,
};
use crossbeam::{sync::WaitGroup, Receiver as CReceiver, Sender as CSender};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

/// high level function to search path
pub fn collection_search(
    collection_handler: Arc<Mutex<Collection>>,
    search_path: Arc<Mutex<SearchPath>>,
    sender_handler: Arc<Mutex<CSender<UiUpdateMsg>>>,
    has_ui: bool,
) -> IFInternalCollectionOutputData {
    let output_data = IFInternalCollectionOutputData::new();
    let output_data_handle = Arc::new(Mutex::new(output_data));
    let output_data_handle2 = output_data_handle.clone();

    let handle_container = Arc::new(Mutex::new(Container::new()));

    let current_search_path = search_path.lock().unwrap().read();
    // start the parallel search threads with rayon, each path its own
    &current_search_path
        .par_iter()
        .enumerate()
        .for_each(|(index, elem)| {
            let sender_loop = sender_handler.clone();
            let collection_data_in_iterator = collection_handler.clone();
            let single_path_collection_data = data::search_in_single_path(
                has_ui,
                handle_container.clone(),
                collection_data_in_iterator,
                sender_loop,
                index,
                elem,
            );
            // accumulate data
            {
                let mut locker = output_data_handle.lock().unwrap();
                locker.nr_found_songs += single_path_collection_data.nr_found_songs;
                locker.nr_internal_duplicates += single_path_collection_data.nr_internal_duplicates;
                locker.nr_searched_files += single_path_collection_data.nr_searched_files;
            }
        });
    let out = &*output_data_handle2.lock().unwrap();
    out.clone()
}

/// high level function to startup net functionality
pub async fn net_search(
    wait_net: WaitGroup,
    net_system_messages: Option<CSender<UiUpdateMsg>>,
    ipc_receive: CReceiver<IPC>,
) -> Result<(), Box<dyn Error>> {
    // This thread will not end itself
    // - can be terminated by ui message
    // - collector finished (depending on definition)

    info!("net started!!");
    let mut network = Net {};

    // startup net thread synchronization
    wait_net.wait();

    network.lookup(net_system_messages, ipc_receive).await
}
