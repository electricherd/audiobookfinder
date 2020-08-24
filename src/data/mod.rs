//! The oldest module, the data module stores all the data needed to collect
//! and also the search, dir algorithms
mod bktree;
pub mod collection;
pub mod ipc;
mod tag_readers;

use self::collection::{Collection, Container};
use super::ctrl::{CollectionPathAlive, ForwardNetMessage, NetMessages, Status, UiUpdateMsg};
use std::{
    path::Path, // path, clear
    sync::{mpsc::Sender, Arc as SArc, Mutex as SMutex},
};

/// Clean paths checks that paths exists and that intersection
/// paths are exluded, also down-up-climbing of existing paths
/// hierarchy are working!
///
/// E.g. given:
///     "/home/user/Music/audiobooks/E-F"
///     "/home/user/Music/audiobooks/E-F/George Orwell"
///     "/home/user/Music/audiobooks/A-D/../../audiobooks/A-D"
///     "/home/user/Music/audiobooks/A-D"
///     "/home/user/Music/audiobooks/E-F/George Orwell/Animal Farm"
///
/// will lead to:
///     "/home/user/Music/audiobooks/E-F"
///     "/home/user/Music/audiobooks/A-D"
///
///
/// todo: implement vfs then write document_test
pub fn clean_paths(unchecked_paths: &Vec<String>) -> Vec<String> {
    let mut checked_paths: Vec<String> = vec![];
    for unchecked in unchecked_paths {
        if let Ok(path_ok) = Path::new(unchecked).canonicalize() {
            if path_ok.is_dir() {
                if let Some(path_str) = path_ok.to_str() {
                    let mut is_add_worthy = true;
                    let path_string = path_str.to_string();
                    for checked in checked_paths.iter_mut() {
                        // check all already checked path, if there is
                        // no reason to not add it, add it.
                        let path_len = path_str.len();
                        let checked_len = checked.len();
                        if path_len < checked_len {
                            // if substring matches checked must be exchanged
                            if checked[..path_len] == path_string {
                                *checked = path_string;
                                is_add_worthy = false;
                                break;
                            }
                        } else {
                            // only add if not substring with any
                            if path_string[..checked_len] == *checked {
                                //
                                is_add_worthy = false;
                                break;
                            }
                        }
                    }
                    // add it if there is no reason not to
                    if is_add_worthy {
                        checked_paths.push(path_str.to_string());
                    }
                } else {
                    warn!(
                        "Path {:?} has some encoding problem, and will not be included in search!",
                        unchecked
                    )
                }
            } else {
                warn!("Path {:?} does not exist as directory/folder, and will not be included in search!", unchecked)
            }
        } else {
            error!(
                "Path {:?} is not a valid directory/folder, and will not be included in search!",
                unchecked
            );
        }
    }
    checked_paths
}

// todo: hide it
pub struct CollectionOutputData {
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
            CollectionOutputData {
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
            CollectionOutputData {
                nr_found_songs: 0,
                nr_duplicates: 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // todo: use crate vfs for better unit tests
    static HAS_FS_UP: bool = false;
    // run: mkdir -p "/tmp/adbf/Music/audiobooks/A-D"
    //      mkdir -p "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm"
    //      mkdir -p "/tmp/adbf/audible/George Orwell/Animal Farm"
    //      mkdir -p "/tmp/adbf/audible/Philip K. Dick/Electric Dreams"
    //      mkdir -p "/tmp/adbf"
    static _TEST_DATA: [&str; 6] = [
        "/tmp/adbf/Music/audiobooks/A-D",
        "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm",
        "/tmp/adbf/audible/George Orwell/Animal Farm",
        "/tmp/adbf/audible/..",
        "/tmp/adbf/audible/Philip K. Dick/Electric Dreams",
        "/tmp/adbf",
    ];

    #[test]
    fn test_clean_paths_overlap() {
        init();

        // 1 out
        let testing_path = vec![
            "/tmp/adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F".to_string(),
        ];
        let return_value = clean_paths(&testing_path);
        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec![
                    "/tmp/adbf/Music/audiobooks/A-D".to_string(),
                    "/tmp/adbf/Music/audiobooks/E-F".to_string(),
                    "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
                ]
            );
        }

        // 2 out
        let testing_path = vec![
            "/tmp/adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F".to_string(),
            "/tmp/adbf/audible/George Orwell".to_string(),
        ];
        let return_value = clean_paths(&testing_path);

        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec![
                    "/tmp/adbf/Music/audiobooks/A-D".to_string(),
                    "/tmp/adbf/Music/audiobooks/E-F".to_string(),
                    "/tmp/adbf/audible/George Orwell".to_string(),
                ]
            );
        }
    }

    #[test]
    fn test_clean_paths_parent() {
        init();

        // ..
        let testing_path = vec![
            "/tmp/adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks/../audiobooks/E-F/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
        ];
        let return_value = clean_paths(&testing_path);
        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec![
                    "/tmp/adbf/Music/audiobooks/A-D".to_string(),
                    "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
                    "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
                ]
            );
        }

        // ..
        let testing_path = vec![
            "/tmp/adbf/Music/../../adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
        ];
        let return_value = clean_paths(&testing_path);
        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec!["/tmp/adbf/Music/audiobooks".to_string(),]
            );
        }

        // ..
        let testing_path = vec![
            "/tmp/adbf/Music/../../adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
        ];
        let return_value = clean_paths(&testing_path);
        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec![
                    "/tmp/adbf/Music/audiobooks/A-D".to_string(),
                    "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
                ]
            );
        }
    }
}
