//! A simple first program in rust, actually I plan to do something even
//! useful. Trying to find all my audiobooks on many machines, identify them,
//! find duplicates (later trying to solve the problem, also including their permissions,
//! different names, but same albums, etc, get all stats about it).
//! It acts as a wrapper around adbflib, which holds major parts of the implemention.
extern crate adbflib;
extern crate rayon;

use adbflib::{
    command_line,
    ctrl::{Ctrl, UiUpdateMsg},
    data::{
        self,
        collection::{Collection, Container},
        ipc::IPC,
        CollectionOutputData,
    },
    logit,
    net::{key_keeper, Net},
};

use async_std::task;
use crossbeam::{channel::unbounded, sync::WaitGroup};
use log::{error, info, trace};
use num_cpus;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    io::{self, Error},
    sync::{mpsc::channel, Arc as SArc, Mutex as SMutex},
};

/// The main application which is central part of communicating with
/// the adbflib, which is closely connected.
fn main() -> io::Result<()> {
    // get start values from the input parser!!!
    let (ui_paths, has_tui, has_webui, has_net, keep_alive, open_browser, web_port, has_ui) =
        command_line::get_start_values();
    // clone as ro
    let ui_path_ro = ui_paths.clone();

    // define collection thread pool
    let assumed_number_of_threads_used =
        if has_webui { 1 } else { 0 } + if has_tui { 1 } else { 0 } + if has_net { 1 } else { 0 };
    let nr_cpus = num_cpus::get();
    let nr_threads_for_collection = if assumed_number_of_threads_used >= nr_cpus {
        1
    } else {
        nr_cpus - assumed_number_of_threads_used
    };
    rayon::ThreadPoolBuilder::new()
        .thread_name(|nr| format!("col_{}", nr))
        .num_threads(nr_threads_for_collection)
        .build_global()
        .unwrap();

    // for synced start of different threads
    let wait_collector = WaitGroup::new();
    let wait_ui = wait_collector.clone();
    let wait_net = wait_collector.clone();

    //
    // prepare the message system
    //
    let (tx, rx) = channel::<UiUpdateMsg>();

    // these will be taken directly
    let tx_net = tx.clone();

    // for now this will stay wrapped
    let tx_from_collector_to_ui = SArc::new(SMutex::new(tx.clone()));

    // start the logging
    logit::Logit::init(logit::Log::File);

    // all optional components are wrapped into threads
    // 1 - UI         ui_thread   (optional)
    // 2 - Net        net_thread  (optional)
    // 3 - Collector  no thread yet (uses multiple rayon worker threads)
    let ui_thread = std::thread::Builder::new()
        .name("ui".into())
        .spawn(move || {
            if has_ui {
                Ctrl::run(
                    key_keeper::get_p2p_server_id(),
                    &ui_path_ro,
                    rx,
                    has_net,
                    wait_ui,
                    has_webui,
                    has_tui,
                    open_browser,
                    web_port,
                )
            } else {
                info!("no ui was created!");
                drop(wait_ui);
                Ok::<(), std::io::Error>(())
            }
        })?;

    let (ipc_send, ipc_receive) = unbounded::<IPC>();

    // 2 - Net Future
    let net_thread = std::thread::Builder::new()
        .name("net".into())
        .spawn(move || {
            if has_net {
                task::block_on(async move {
                    // This thread will not end itself
                    // - can be terminated by ui message
                    // - collector finished (depending on definition)

                    info!("net started!!");
                    let net_system_messages = tx_net;
                    let mut network = Net::new(has_ui, net_system_messages);

                    // startup net synchronization
                    wait_net.wait();

                    network.lookup(ipc_receive).await;
                    info!("net finished!!");
                    Ok::<(), Error>(())
                })
            } else {
                info!("no net!");
                drop(wait_net);
                Ok::<(), Error>(())
            }
        })?;

    // the collector ... still a problem with threading and parse_args
    // borrowing?? but since rayon is used, using a separate thread is not really
    // important
    // but yet this simple bracket to enclose this a little
    {
        trace!("syncing with 2 other threads");
        wait_collector.wait();
        trace!("sync with net and ui done ... collector can start");

        let synced_to_ui_messages = tx_from_collector_to_ui.clone();

        // start the parallel search threads with rayon, each path its own
        let init_collection =
            Collection::new(&key_keeper::get_p2p_server_id(), nr_threads_for_collection);
        let collection_protected = SArc::new(SMutex::new(init_collection));

        let output_data = SArc::new(SMutex::new(CollectionOutputData { nr_found_songs: 0 }));
        let handle_container = SArc::new(SMutex::new(Container::new()));

        &ui_paths.par_iter().enumerate().for_each(|(index, elem)| {
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
            // short lock to accumulate data
            {
                output_data.lock().unwrap().nr_found_songs +=
                    single_path_collection_data.nr_found_songs;
            }
        });

        info!("collector finished!!");
        if !has_ui {
            collection_protected
                .lock()
                .and_then(|locked_collection| Ok(locked_collection.print_stats()))
                .unwrap_or(())
        }
        if has_net {
            let to_send = output_data.lock().unwrap().nr_found_songs;
            ipc_send
                .send(IPC::DoneSearching(to_send))
                .unwrap_or_else(|_| {
                    error!("net has to be up and receiving this send!");
                });
        }
    }

    // look for keeping alive argument if that was chosen
    if keep_alive {
        ui_thread
            .join()
            .unwrap_or_else(|_| {
                // it doesn't matter because we will terminate anyway
                error!("this should be result of ui thread!!");
                Ok(())
            })
            .unwrap();
        if has_ui {
            info!("Stopped ui thread");
        }
        // net thread
        if has_ui {
            // if had a ui , net_thread will stop also after ui quit
            drop(net_thread);
        } else {
            // if didn't have ui, net_thread will continue running
            println!(
                "Search is finished, but net thread is kept running!\nTo stop send break command (ctrl-c)!"
            );
            net_thread
                .join()
                .unwrap_or_else(|_| {
                    // it doesn't matter because we will terminate anyway
                    error!("is this normal when joining net thread???");
                    Ok(())
                })
                .unwrap();
            info!("Stopped net thread");
        }
    } else {
        drop(net_thread);
        drop(ui_thread);
    }
    Ok(())
}
