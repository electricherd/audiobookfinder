//! A simple first program in rust, actually I plan to do something even
//! useful. Trying to find all my audiobooks on many machines, identify them,
//! find duplicates (later trying to solve the problem, also including their permissions, different names, but same albums, etc),
//! get all stats about it).
//! It acts as a wrapper around adbflib, which holds major parts of the implemention.
extern crate adbflib;
extern crate clap;
extern crate rayon;

use adbflib::{
    ctrl::{Ctrl, NetAlive, ReceiveDialog, Status, SystemMsg},
    data::{self, Collection},
    logit,
    net::{key_keeper, Net},
};
use async_std::{
    sync::{Arc, Mutex},
    task,
};
use futures::try_join;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    io::{self, Error, ErrorKind},
    path::Path, // path, clear
    sync::{
        self,
        mpsc::{channel, Sender},
    },
};

static INPUT_FOLDERS: &str = "folders";
static APP_TITLE: &str = concat!("The audiobook finder (", env!("CARGO_PKG_NAME"), ")");
static ARG_NET: &str = "net";
static ARG_TUI: &str = "tui";
static ARG_WEBUI: &str = "webui";

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const HOMEPAGE: &'static str = env!("CARGO_PKG_HOMEPAGE");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

fn main() -> io::Result<()> {
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

    // either one will have a ui, representing data and error messages
    let has_ui = has_tui || has_webui;

    //
    // prepare the message system
    //
    let (tx, rx) = channel::<SystemMsg>();

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

    // start the main operators
    //
    task::block_on(async move {
        // all optional components are wrapped into a future which can result in an empty future,
        // but collector future will not be empty
        // 1 - UI         ui_future   (optional)
        // 2 - Net        net_future  (optional)
        // 3 - Collector  collector_future

        let ui_future = async move {
            if has_ui {
                match Ctrl::new(
                    key_keeper::get_p2p_server_id(),
                    &ui_paths,
                    rx,
                    tx.clone(),
                    has_net,
                ) {
                    Ok(mut controller) => {
                        if has_webui {
                            controller.run_webui().await?;
                        }
                        if has_tui {
                            println!("starting tui");
                            // do finally the necessary
                            // this blocks this async future
                            controller
                                .run_tui()
                                .await
                                .map_err(|error_text| Error::new(ErrorKind::Other, error_text))?;
                        }
                        Ok::<(), Error>(())
                    }
                    Err(error_text) => {
                        println!("{:?}", error_text);
                        Err(Error::new(ErrorKind::Other, error_text))
                    }
                }
            } else {
                println!("no ui was created!");
                Ok::<(), Error>(())
            }
        };

        // 2 - Net Future
        let net_future = async move {
            // This thread will not end itself
            // - can be terminated by ui message
            // - collector finished (depending on definition)
            if has_net {
                println!("Net Started!!");
                let net_system_messages = tx_net;
                let mut network = Net::new(
                    key_keeper::get_p2p_server_id(),
                    has_tui,
                    net_system_messages,
                );
                network.lookup().await;
                println!("Net finished!!");
                Ok::<(), Error>(())
            } else {
                println!("no net!");
                Ok::<(), Error>(())
            }
        };

        // start the parallel search threads with rayon, each path its own
        let collector_future = async move {
            // initialize the data collection for all
            let init_collection = Collection::new(&key_keeper::get_p2p_server_id(), max_threads);
            let collection_protected = sync::Arc::new(sync::Mutex::new(init_collection));

            // todo: is this right? async lock to create a sync message mutex?
            let synced_to_ui_messages = tx_from_collector_to_ui.lock().await.clone();
            let wrapped_to_ui_sender = sync::Arc::new(sync::Mutex::new(synced_to_ui_messages));

            all_pathes.par_iter().enumerate().for_each(|(index, elem)| {
                let collection_data_in_iterator = collection_protected.clone();
                let to_ui_messages_in_iterator = wrapped_to_ui_sender.clone();
                search_in_single_path(
                    has_ui,
                    has_tui,
                    collection_data_in_iterator,
                    to_ui_messages_in_iterator,
                    index,
                    elem,
                );
            });
            if !has_ui {
                collection_protected
                    .lock()
                    .and_then(|locked_collection| Ok(locked_collection.print_stats()))
                    .unwrap_or(())
                // todo: send terminate to net runner depending if it should continue or not
            }
            Ok::<(), Error>(())
        };

        // Compose all futures that possible threads inside are running
        try_join!(ui_future, net_future, collector_future)
    })
    .and_then(|(_, _, _)| {
        // shrink the 3 ok results to one
        println!("done here");
        Ok(())
    })
}

fn search_in_single_path(
    has_ui: bool,
    has_tui: bool,
    collection_protected: sync::Arc<sync::Mutex<Collection>>,
    mutex_to_ui_msg: sync::Arc<sync::Mutex<Sender<SystemMsg>>>,
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
                    println!("send startAnimation for path {:?}", index);
                    locked_to_start_ui_message
                        .send(SystemMsg::StartAnimation(
                            NetAlive::BusyPath(index),
                            Status::ON,
                        ))
                        .unwrap_or_else(|_| {
                            println!("... lost start animation for index {:?}", index)
                        });
                    Ok(())
                })
                .unwrap_or_else(|_| println!("... that should not happen here at start"));
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
                    println!("send stopAnimation for path {:?}", index);
                    mutex_to_ui_msg
                        .lock()
                        .and_then(|locked_to_stop_ui_message| {
                            locked_to_stop_ui_message
                                .send(SystemMsg::StartAnimation(
                                    NetAlive::BusyPath(index),
                                    Status::OFF,
                                ))
                                .unwrap_or_else(|_| {
                                    println!("... lost stop animation for {:?}", index)
                                });
                            Ok(())
                        })
                        .unwrap_or_else(|_| {
                            println!("... that should not happen here at stop");
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
                    let debug_message_id = ReceiveDialog::Debug;
                    let text = text.to_string();
                    mutex_to_ui_msg
                        .lock()
                        .and_then(|locked_update_message| {
                            locked_update_message
                                .send(SystemMsg::Update(debug_message_id, text))
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
