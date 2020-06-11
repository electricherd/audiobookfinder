//! A simple first program in rust, actually I plan to do something even
//! useful. Trying to find all my audiobooks on many machines, identify them,
//! find duplicates (later trying to solve the problem, also including their permissions, different names, but same albums, etc),
//! get all stats about it).
//! It acts as a wrapper around adbflib, which holds major parts of the implemention.
extern crate adbflib;
extern crate clap;
extern crate rayon;

use adbflib::{
    common::startup::{StartUp, SyncStartUp},
    ctrl::{CollectionPathAlive, Ctrl, NetMessages, Status, UiUpdateMsg},
    data::{self, Collection},
    logit,
    net::{key_keeper, Net},
};
use async_std::task;
use log::{error, info, trace, warn};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    io::{self, Error, ErrorKind},
    path::Path, // path, clear
    sync::{
        self,
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
};

static INPUT_FOLDERS: &str = "folders";
static APP_TITLE: &str = concat!("The audiobook finder (", env!("CARGO_PKG_NAME"), ")");
static ARG_NET: &str = "net";
static ARG_TUI: &str = "tui";
static ARG_WEBUI: &str = "webui";
static ARG_KEEP_ALIVE: &str = "keep";

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const HOMEPAGE: &'static str = env!("CARGO_PKG_HOMEPAGE");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

fn main() -> io::Result<()> {
    // for synced start
    let (ready_ui_send, ready_ui_receiver) = channel::<SyncStartUp>();
    let (ready_net_send, ready_net_receiver) = channel::<SyncStartUp>();

    let parse_args = clap::App::new(APP_TITLE)
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .long_about::<&str>(
            &[
                &DESCRIPTION,
                "\n\
                 It reads data from possibly multiple given path(s). Via local network it searches \
                 for other instances of the program, and will later exchange data securely.\n\
                 All information gathered will be used to find duplicates, versions of \
                 different quality, different tags for same content (spelling, \
                 incompleteness).\n\
                 For documentation see: ",
                &HOMEPAGE,
                "\n \
                 A program to learn, embrace, and love Rust! \n\
                 Have fun!",
            ]
            .concat(),
        )
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets custom config file (not implemented yet)")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(ARG_TUI)
                .short("t")
                .long(ARG_TUI)
                .help("Run with TUI")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_WEBUI)
                .short("w")
                .long(ARG_WEBUI)
                .help("Run with-in a webui.")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_NET)
                .short("n")
                .long(ARG_NET)
                .help("With net search for other audiobookfinders running in local network.")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_KEEP_ALIVE)
                .short("k")
                .long(ARG_KEEP_ALIVE)
                .help("With keep alive process will continue even after search has been performed.")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(INPUT_FOLDERS)
                .help("Sets multiple input folder(s) to be searched for audio files.")
                .multiple(true)
                .required(false),
        )
        .get_matches();
    //
    let max_threads = rayon::current_num_threads();

    // tricky thing, but I really like that
    let all_pathes = if let Some(correct_input) = parse_args.values_of(INPUT_FOLDERS) {
        correct_input.collect()
    } else {
        vec!["."]
    };

    //
    // check argments if tui and net search is needed
    //
    let has_arg = |x: &str| parse_args.is_present(x);

    let has_tui = has_arg(ARG_TUI);
    let has_webui = has_arg(ARG_WEBUI);
    let has_net = has_arg(ARG_NET);
    let keep_alive = has_arg(ARG_KEEP_ALIVE);

    // either one will have a ui, representing data and error messages
    let has_ui = has_tui || has_webui;

    //
    // prepare the message system
    //
    let (tx, rx) = channel::<UiUpdateMsg>();

    // these will be taken directly
    let tx_net = tx.clone();

    // for now this will stay wrapped
    let tx_from_collector_to_ui = Arc::new(Mutex::new(tx.clone()));

    //
    // pathes: copy to vec<&str>
    //
    let ui_paths = all_pathes.iter().map(|s| s.to_string()).collect();

    // start the logging
    logit::Logit::init(logit::Log::File);

    // all optional components are wrapped into threads
    // 1 - UI         ui_thread   (optional)
    // 2 - Net        net_thread  (optional)
    // 3 - Collector  no thread yet (will use multiple rayon worker threads)
    let ui_thread = std::thread::Builder::new()
        .name("ui".into())
        .spawn(move || {
            if has_ui {
                match Ctrl::new(
                    key_keeper::get_p2p_server_id(),
                    &ui_paths,
                    rx,
                    has_net,
                    ready_ui_send,
                ) {
                    Ok(mut controller) => {
                        // wait for synchronisation
                        if has_webui {
                            controller.run_webui()?;
                        }
                        if has_tui {
                            info!("starting tui");

                            // do finally the necessary
                            // this blocks this async future
                            controller
                                .run_tui()
                                .map_err(|error_text| Error::new(ErrorKind::Other, error_text))?;
                        }
                        Ok::<(), Error>(())
                    }
                    Err(error_text) => {
                        error!("{:?}", error_text);
                        Err(Error::new(ErrorKind::Other, error_text))
                    }
                }
            } else {
                info!("no ui was created!");
                ready_ui_send
                    .send(SyncStartUp::NoWait)
                    .expect("collection from ui receiver not yet there???");
                Ok::<(), Error>(())
            }
        });

    // 2 - Net Future
    let net_thread = std::thread::Builder::new()
        .name("net".into())
        .spawn(move || {
            task::block_on(async move {
                // This thread will not end itself
                // - can be terminated by ui message
                // - collector finished (depending on definition)
                if has_net {
                    info!("net started!!");
                    let net_system_messages = tx_net;
                    let mut network = Net::new(
                        key_keeper::get_p2p_server_id(),
                        has_tui,
                        net_system_messages,
                    );

                    // startup net synchronization
                    StartUp::block_on_sync(ready_net_send, "net");

                    network.lookup().await;
                    info!("net finished!!");
                    Ok::<(), Error>(())
                } else {
                    info!("no net!");
                    ready_net_send
                        .send(SyncStartUp::NoWait)
                        .expect("collection from net receiver not yet there???");
                    Ok::<(), Error>(())
                }
            })
            .unwrap_or_else(|_| {
                // do nothing
                ()
            })
        });

    // the collector ... still a problem with threading and parse_args
    // borrowing?? but since rayon is used, using a separate thread is not really
    // important
    // but yet this simple bracket to enclose this a little
    {
        trace!("syncing with 2 other threads");
        // todo: return false is yet weak, it means timeout happened
        StartUp::send_and_block2(&ready_ui_receiver, &ready_net_receiver);
        trace!("sync with net and ui done ... collector can start");

        let synced_to_ui_messages = tx_from_collector_to_ui.clone();

        // start the parallel search threads with rayon, each path its own
        let init_collection = Collection::new(&key_keeper::get_p2p_server_id(), max_threads);
        let collection_protected = sync::Arc::new(sync::Mutex::new(init_collection));

        all_pathes.par_iter().enumerate().for_each(|(index, elem)| {
            let sender_loop = synced_to_ui_messages.clone();
            let collection_data_in_iterator = collection_protected.clone();
            search_in_single_path(
                has_ui,
                has_tui,
                collection_data_in_iterator,
                sender_loop,
                index,
                elem,
            );
        });
        if !has_ui {
            collection_protected
                .lock()
                .and_then(|locked_collection| Ok(locked_collection.print_stats()))
                .unwrap_or(())
        }
        info!("collector finished!!");
    }

    // look for keeping alive argument if that was chosen
    if keep_alive {
        ui_thread
            .and_then(|running_thread| Ok(running_thread.join()))
            .unwrap_or_else(|_| {
                // it doesn't matter because we will terminate anyway
                error!("this should be result of ui thread!!");
                Ok(Ok(()))
            })
            .unwrap_or_else(|_| {
                // it doesn't matter because we will terminate anyway
                error!("is this normal when joining ui thread???");
                Ok(())
            })
            .unwrap();
        if has_ui {
            info!("Stopped ui thread");
        }
        net_thread
            .and_then(|running_thread| Ok(running_thread.join()))
            .unwrap_or_else(|_| {
                // it doesn't matter because we will terminate anyway
                error!("is this normal when joining net thread???");
                Ok(())
            })
            .unwrap();
        if has_net {
            info!("Stopped net thread");
        }
    } else {
        drop(net_thread);
        drop(ui_thread);
    }
    Ok(())
}

fn search_in_single_path(
    has_ui: bool,
    has_tui: bool,
    collection_protected: sync::Arc<sync::Mutex<Collection>>,
    mutex_to_ui_msg: sync::Arc<sync::Mutex<Sender<UiUpdateMsg>>>,
    index: usize,
    elem: &str,
) {
    if !has_ui {
        println!("[{:?}] looking into path {:?}", index, elem);
    } else {
        // send start animation for that path
        if has_tui {
            mutex_to_ui_msg
                .lock()
                .and_then(|locked_to_start_ui_message| {
                    trace!("send startAnimation for path {:?}", index);
                    locked_to_start_ui_message
                        .send(UiUpdateMsg::CollectionUpdate(
                            CollectionPathAlive::BusyPath(index),
                            Status::ON,
                        ))
                        .unwrap_or_else(|_| {
                            warn!("... lost start animation for index {:?}", index)
                        });
                    trace!("start busy animation");
                    Ok(())
                })
                .unwrap_or_else(|_| error!("... that should not happen here at start"));
        }
    }
    let locked_collection = &mut *collection_protected.lock().unwrap();

    // do it: main task here is to visit and dive deep
    //        into the subfolders of this folder
    match locked_collection.visit_dirs(Path::new(elem), &data::Collection::visit_files) {
        Ok(local_stats) => {
            if has_ui {
                // send stop animation for that path
                if has_tui {
                    trace!("send stopAnimation for path {:?}", index);
                    mutex_to_ui_msg
                        .lock()
                        .and_then(|locked_to_stop_ui_message| {
                            locked_to_stop_ui_message
                                .send(UiUpdateMsg::CollectionUpdate(
                                    CollectionPathAlive::BusyPath(index),
                                    Status::OFF,
                                ))
                                .unwrap_or_else(|_| {
                                    warn!("... lost stop animation for {:?}", index)
                                });
                            trace!("stop busy animation");
                            Ok(())
                        })
                        .unwrap_or_else(|_| {
                            error!("... that should not happen here at stop");
                        });
                }
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
        }
        Err(_e) => {
            let text = format!("An error has occurred in search path [{}]!!", index);
            if has_ui {
                if has_tui {
                    let debug_message_id = NetMessages::Debug;
                    mutex_to_ui_msg
                        .lock()
                        .and_then(|locked_update_message| {
                            locked_update_message
                                .send(UiUpdateMsg::NetUpdate((debug_message_id, text)))
                                .unwrap();
                            Ok(())
                        })
                        .unwrap();
                }
            } else {
                println!("{:?}", text);
            }
        }
    }
}
