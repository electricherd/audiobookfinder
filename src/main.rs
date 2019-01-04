//! A simple first program in rust, actually I plan to do something even
//! useful. Trying to find all my audiobooks on many machines, identify them,
//! find duplicates (later trying to solve the problem, also including their permissions, different names, but same albums, etc),
//! get all stats about it).
//! It acts as a wrapper around adbflib, which holds major parts of the implemention.
extern crate clap;
extern crate hostname;
extern crate rayon;
extern crate uuid;

extern crate adbflib;

use adbflib::{ctrl::{Alive, Ctrl, ReceiveDialog, Status, SystemMsg},
              data::{self, Collection},
              logit,
              net::Net};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{path::Path, // path, clear
          sync::{mpsc, Arc, Mutex},
          thread};
use uuid::Uuid;

static INPUT_FOLDERS: &str = "folders";
static APP_TITLE: &str = concat!("The audiobook finder (", env!("CARGO_PKG_NAME"), ")");
static ARG_NET: &str = "net";
static ARG_TUI: &str = "tui";

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const HOMEPAGE: &'static str = env!("CARGO_PKG_HOMEPAGE");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let parse_args = clap::App::new(APP_TITLE)
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .long_about::<&str>(&[
            &DESCRIPTION,
            "\n\
             It reads data\
             from possibly multiple given path(s). Via local network it searches for other \
             instances of the program, and will later exchange data securely.\n\
             All information gathered will be used to find duplicates, versions of \
             different quality, different tags for same content (spelling, \
             incompleteness).\n\
             For documentation see: ",
            &HOMEPAGE,
            "\n \
             A program to learn, embrace, and love Rust! \n\
             Have fun!",
        ].concat())
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
                .long("TUI")
                .help("Run with TUI")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_NET)
                .short("n")
                .long("net")
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

    // tricky thing, but I really like that
    let all_pathes = if let Some(correct_input) = parse_args.values_of(INPUT_FOLDERS) {
        correct_input.collect()
    } else {
        vec!["."]
    };

    let hostname = hostname::get_hostname().unwrap_or("undefined".to_string());
    let max_threads = rayon::current_num_threads();

    // check if tui and net search is needed
    let has_arg = |x: &str| parse_args.is_present(x);

    // has to be mutable because in case of error this might be changed
    let mut has_tui = has_arg(ARG_TUI);
    let has_net = has_arg(ARG_NET);

    // prepare the message system
    let (tx, rx) = mpsc::channel::<SystemMsg>();

    let tx_sys_mut = Mutex::new(tx.clone());
    let tx_net_mut = Mutex::new(tx.clone());
    let tx_net_alive_mut = Mutex::new(tx.clone());

    // copy to vec<&str>
    let tui_pathes = all_pathes.iter().map(|s| s.to_string()).collect();

    // get an unique id for this client
    let client_id = Uuid::new_v4();

    // the signal to tell further processes, that tui creation worked
    // and the messages actually go somewhere, otherwise it will assume: no tui
    let (send_tui_worked, receiver_tui_worked) = mpsc::channel::<bool>();

    // start the tui thread
    let tui_runner = thread::spawn(move || {
        if has_tui {
            // start animation .... timer and so on
            if let Ok(starter) = tx_net_alive_mut.lock() {
                starter
                    .send(SystemMsg::StartAnimation(Alive::HostSearch, Status::ON))
                    .unwrap();
            }
            let controller = Ctrl::new(client_id, &tui_pathes, rx, tx.clone(), has_net);
            match controller {
                Ok(mut controller) => {
                    // signal ok
                    send_tui_worked.send(true).unwrap();
                    drop(send_tui_worked);

                    // do finally the necessary
                    controller.run();
                }
                Err(error_text) => {
                    println!("{:?}", error_text);
                    // no tui could be created
                    send_tui_worked.send(false).unwrap();
                    drop(send_tui_worked);
                }
            }
        }
    });

    // blocks that one signal (but that should be very short time)
    if has_tui {
        if let Ok(tui_check_receiver) = receiver_tui_worked.recv() {
            has_tui = tui_check_receiver;
        } else {
            println!("Something bad has happenend!!!");
            has_tui = false;
        }
    }
    // make it immutable from now on
    let has_tui = has_tui;

    // start the logging
    logit::Logit::init(logit::Log::File);

    // start the net runner thread
    let tx_net_mut_arc = Arc::new(tx_net_mut);
    let net_runner = thread::spawn(move || {
        if has_net {
            if let Ok(mut network) = Net::new(
                client_id,
                has_tui,
                // need to simplify and clarify this here ......
                // but this lock unwrap is safe
                tx_net_mut_arc.lock().unwrap().clone(),
            ) {
                if network.start_com_server().is_ok() {
                    network.lookup();
                }
            }
        }
    });

    // initialize the data collection for all
    let init_collection = Collection::new(hostname, &client_id, max_threads);
    let collection_protected = Arc::new(Mutex::new(init_collection));

    // start the search threads, each path its own
    all_pathes.par_iter().enumerate().for_each(|(index, elem)| {
        if !has_tui {
            println!("[{:?}] looking into path {:?}", index, elem);
        } else {
            // start animation .... timer and so on
            if let Ok(starter) = tx_sys_mut.lock() {
                starter
                    .send(SystemMsg::StartAnimation(
                        Alive::BusyPath(index),
                        Status::ON,
                    ))
                    .unwrap();
            }
        }
        let live_here = collection_protected.clone();
        let locked_collection = live_here.lock();
        if let Ok(mut pure_collection) = locked_collection {
            match pure_collection.visit_dirs(Path::new(elem), &data::Collection::visit_files) {
                Ok(local_stats) => {
                    if has_tui {
                        // stop animation
                        if let Ok(stopper) = tx_sys_mut.lock() {
                            stopper
                                .send(SystemMsg::StartAnimation(
                                    Alive::BusyPath(index),
                                    Status::OFF,
                                ))
                                .unwrap();
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
                    if has_tui {
                        let debug_message_id = ReceiveDialog::Debug;
                        let text = text.to_string();
                        let debug_text = tx_sys_mut.lock();
                        if let Ok(debug_text) = debug_text {
                            debug_text
                                .send(SystemMsg::Update(debug_message_id, text))
                                .unwrap();
                        }
                    } else {
                        println!("{:?}", text);
                    }
                }
            }
        }
    });

    if !has_tui {
        if let Ok(result_collection) = collection_protected.lock() {
            result_collection.print_stats();
        }
        let _ = net_runner.join();
    } else {
        let _ = tui_runner.join();
        // if tui, net runner shall stop when tui stops
        drop(net_runner);
    }

    println!("Finished!");
}
