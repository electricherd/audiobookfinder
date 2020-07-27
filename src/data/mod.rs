//! The oldest module, the data module stores all the data needed to collect
//! and also the search, dir algorithms
pub mod bktree;
pub mod collection;
pub mod ipc;

use super::ctrl::{CollectionPathAlive, ForwardNetMessage, NetMessages, Status, UiUpdateMsg};
use collection::{Collection, Container};

use std::{
    path::Path, // path, clear
    sync::{mpsc::Sender, Arc as SArc, Mutex as SMutex},
};

// todo: hide it
pub struct CollectionOutputData {
    pub nr_found_songs: u32,
}

pub fn search_in_single_path(
    has_ui: bool,
    collection_data: SArc<SMutex<Container>>,
    collection_protected: SArc<SMutex<Collection>>,
    mutex_to_ui_msg: SArc<SMutex<Sender<UiUpdateMsg>>>,
    index: usize,
    elem: &str,
) -> CollectionOutputData {
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
    // todo: crap : unlock here is stupid!
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
                             searched: {se:>width$}, other: {ot:>width$}",
                    an = local_stats.analyzed,
                    fa = local_stats.faulty,
                    se = local_stats.searched,
                    ot = local_stats.other,
                    width = 3
                );
                println!("[{:?}] done {}", index, text);
            }
            // return this here
            CollectionOutputData {
                nr_found_songs: local_stats.analyzed,
            }
        }
        Err(_e) => {
            let text = format!("An error has occurred in search path [{}]!!", index);
            if has_ui {
                mutex_to_ui_msg
                    .lock()
                    .and_then(|locked_update_message| {
                        locked_update_message
                            .send(UiUpdateMsg::NetUpdate(ForwardNetMessage::Stats(
                                NetMessages::Debug(text),
                            )))
                            .unwrap();
                        Ok(())
                    })
                    .unwrap();
            } else {
                println!("{:?}", text);
            }
            // return this here
            CollectionOutputData { nr_found_songs: 0 }
        }
    }
}
