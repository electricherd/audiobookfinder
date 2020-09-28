//! The adbflib is a LIB and a facade to many modules that interact with
//! the audiobookfinder program.
//! In this file all crates are name before the modules which use them.
#![crate_name = "adbfbinlib"]
#![crate_type = "lib"]
// ALL
pub mod common;
pub mod ctrl;
pub mod data;
pub mod net;

//logger
#[macro_use]
extern crate log;

// data
#[macro_use]
extern crate serde_derive;

// config
#[macro_use]
extern crate lazy_static;

use crate::{
    common::paths::SearchPath,
    ctrl::UiUpdateMsg,
    data::{audio_info::Container, collection::Collection, IFInternalCollectionOutputData},
};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    error, fmt, io,
    sync::{mpsc::channel, Arc as SArc, Mutex as SMutex},
};

/// A useless Error just for the Demo
#[derive(Copy, Clone, Debug)]
pub struct AdbflibError;
impl fmt::Display for AdbflibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error with adbfbinlib this page.")
    }
}
impl error::Error for AdbflibError {}
impl From<io::Error> for AdbflibError {
    fn from(_: io::Error) -> Self {
        Self
    }
}

pub async fn file_count_good(input_path: Vec<String>) -> Result<u32, AdbflibError> {
    // todo: why does this just work?
    Ok(file_count_good_wrapped(input_path))
}

fn file_count_good_wrapped(input_path: Vec<String>) -> u32 {
    // prepare data
    let cleaned_paths = SearchPath::new(&input_path);

    let search_path = SArc::new(SMutex::new(cleaned_paths));
    let (tx, _rx) = channel::<UiUpdateMsg>();
    let synced_to_ui_messages = SArc::new(SMutex::new(tx.clone()));
    let has_ui = false;

    // start the parallel search threads with rayon, each path its own
    let init_collection = Collection::new();
    let collection_protected = SArc::new(SMutex::new(init_collection));

    let output_data_handle = SArc::new(SMutex::new(IFInternalCollectionOutputData::new()));
    let output_data_return_handle = output_data_handle.clone();

    let handle_container = SArc::new(SMutex::new(Container::new()));

    let current_search_path = search_path.lock().unwrap().read();
    &current_search_path
        .par_iter()
        .enumerate()
        .for_each(|(index, elem)| {
            let sender_loop = synced_to_ui_messages.clone();
            let collection_data_in_iterator = collection_protected.clone();
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

    // scope and block trickery for lifetime and mutability
    let return_value;
    // lock for borrowed return_value
    {
        return_value = output_data_return_handle.lock().unwrap().nr_found_songs;
    }
    return_value
}
