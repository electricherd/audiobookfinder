//! The oldest module, the data module stores all the data needed to collect
//! and also the search, dir algorithms
mod bktree;
pub mod collection;
pub mod ipc;
mod tag_readers;

use self::collection::{Collection, Container};
use super::ctrl::{CollectionPathAlive, ForwardNetMsg, NetInfoMsg, Status, UiUpdateMsg};
use std::{
    path::Path, // path, clear
    sync::{mpsc::Sender, Arc as SArc, Mutex as SMutex},
};

/// Interface of what collection output data will return
pub struct InterfaceCollectionOutputData {
    pub nr_found_songs: u32,
    pub nr_duplicates: u32,
}

/// The collection search! Searches on the file system a concrete, single path.
pub fn search_in_single_path(
    has_ui: bool,
    collection_data: SArc<SMutex<Container>>,
    collection_protected: SArc<SMutex<Collection>>,
    mutex_to_ui_msg: SArc<SMutex<Sender<UiUpdateMsg>>>,
    index: usize,
    elem: &str,
) -> InterfaceCollectionOutputData {
    if !has_ui {
        println!("[{:?}] looking into path {:?}", index, elem);
    } else {
        // send start animation for that path
        mutex_to_ui_msg
            .lock()
            .and_then(|locked_to_start_ui_message| {
                trace!("send startAnimation for path {:?}", index);
                locked_to_start_ui_message
                    .send(UiUpdateMsg::CollectionUpdate(
                        CollectionPathAlive::BusyPath(index),
                        Status::ON,
                    ))
                    .unwrap_or_else(|_| warn!("... lost start animation for index {:?}", index));
                trace!("start busy animation");
                Ok(())
            })
            .unwrap_or_else(|_| error!("... that should not happen here at start"));
    }
    // this collection lock is fine, it's not the main Container, only some data
    let locked_collection = &mut *collection_protected.lock().unwrap();

    // do it: main task here is to visit and dive deep
    //        into the subfolders of this folder
    match locked_collection.visit_path(
        collection_data,
        Path::new(elem),
        &collection::Collection::visit_files,
    ) {
        Ok(local_stats) => {
            if has_ui {
                // send stop animation for that path
                trace!("send stopAnimation for path {:?}", index);
                mutex_to_ui_msg
                    .lock()
                    .and_then(|locked_to_stop_ui_message| {
                        locked_to_stop_ui_message
                            .send(UiUpdateMsg::CollectionUpdate(
                                CollectionPathAlive::BusyPath(index),
                                Status::OFF,
                            ))
                            .unwrap_or_else(|_| warn!("... lost stop animation for {:?}", index));
                        trace!("stop busy animation");
                        Ok(())
                    })
                    .unwrap_or_else(|_| {
                        error!("... that should not happen here at stop");
                    });
            } else {
                let text = format!(
                    "\n\
                             analyzed: {an:>width$}, faulty: {fa:>width$}\n\
                             duplicates: {du:>width$}\n\
                             searched: {se:>width$}, other: {ot:>width$}",
                    an = local_stats.analyzed,
                    fa = local_stats.faulty,
                    du = local_stats.duplicates,
                    se = local_stats.searched,
                    ot = local_stats.other,
                    width = 3
                );
                println!("[{:?}] done {}", index, text);
            }
            // return this here
            InterfaceCollectionOutputData {
                nr_found_songs: local_stats.analyzed,
                nr_duplicates: local_stats.duplicates,
            }
        }
        Err(_e) => {
            let text = format!("An error has occurred in search path [{}]!!", index);
            if has_ui {
                mutex_to_ui_msg
                    .lock()
                    .and_then(|locked_update_message| {
                        locked_update_message
                            .send(UiUpdateMsg::NetUpdate(ForwardNetMsg::Stats(
                                NetInfoMsg::Debug(text),
                            )))
                            .unwrap();
                        Ok(())
                    })
                    .unwrap();
            } else {
                println!("{:?}", text);
            }
            // return this here
            InterfaceCollectionOutputData {
                nr_found_songs: 0,
                nr_duplicates: 0,
            }
        }
    }
}
