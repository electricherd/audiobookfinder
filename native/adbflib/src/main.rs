//! A simple first program in rust, actually I plan to do something even
//! useful. Trying to find all my audiobooks on many machines, identify them,
//! find duplicates (later trying to solve the problem, also including their permissions,
//! different names, but same albums, etc, get all stats about it).
//! It acts as a wrapper around adbflib, which holds major parts of the implemention.
//!
//! For debugging, use e.g.
//!   ADBF_LOG_LVL=file RUST_BACKTRACE=full RUST_LOG=adbfbinlib::webui=trace
//!
//!               ADBF_LOG_LVL=file   : is the method
//!               RUST_BACKTRACE=full : is Rust's trace level
//!  RUST_LOG=adbfbinlib::webui=trace : is program's inner debugging mod
//!

mod command_line;

use adbfbinlib::{
    common::{logit, paths::SearchPath},
    ctrl::{Ctrl, UiUpdateMsg},
    data::{
        collection::Collection,
        ipc::{IFCollectionOutputData, IPC},
    },
    net::subs::key_keeper,
    shared,
};
use async_std::task;
use crossbeam::{channel::unbounded, sync::WaitGroup};
use ctrlc;
use exitcode;
use log::{error, info, trace};
use num_cpus;
use std::{
    cmp, env, io, process,
    sync::{Arc as SArc, Mutex as SMutex},
};

/// The main application which is central part of communicating with
/// the adbflib, which is closely connected.
fn main() -> io::Result<()> {
    // get start values from the input parser!!!
    let (ui_paths, has_tui, has_webui, has_net, keep_alive, open_browser, web_port, has_ui) =
        command_line::get_start_values();

    // read into paths
    let cleaned_paths = SearchPath::new(&ui_paths);
    if cleaned_paths.len() != ui_paths.len() && !has_tui && !has_webui && !open_browser {
        println!("Some paths/folders intersect and will not be used!");
    }

    // only use the arc
    let search_path = SArc::new(SMutex::new(cleaned_paths));
    let search_path_ui = search_path.clone();

    // define collection thread pool
    let ctrlc_thread = 1;
    let assumed_number_of_threads_used = if has_webui { 1 } else { 0 }
        + if has_tui { 1 } else { 0 }
        + if has_net { 1 } else { 0 }
        + ctrlc_thread;
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
    let (tx, rx) = unbounded::<UiUpdateMsg>();

    // these will be taken directly
    let tx_net = tx.clone();

    // ui message from here (collection)
    let tx_col = tx.clone();

    // for now this will stay wrapped
    let tx_from_collector_to_ui = SArc::new(SMutex::new(tx.clone()));

    // start the logging
    logit::Logit::init(logit::read_env_level(
        &env::var("ADBF_LOG").unwrap_or("".into()),
    ));

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
                    search_path_ui,
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

    // create a crossbeam channel for massive data exchange
    // with data I called IPC ...
    // in contrast to normal channels crossbeam channels are
    // n:m, so senders can be cloned as well!
    let (ipc_send, ipc_receive) = unbounded::<IPC>();

    // 2 - Net Future
    let net_thread = std::thread::Builder::new()
        .name("net".into())
        .spawn(move || {
            if has_net {
                let sender = Some(tx_net);
                task::block_on(
                    async move { shared::net_search(wait_net, sender, ipc_receive).await },
                )
                .map_err(|error| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("network error: {}", error),
                    )
                })
            } else {
                info!("no net!");
                drop(wait_net);
                Ok::<(), std::io::Error>(())
            }
        })?;

    // CTRL-C exit handler (is wrapped inside a thread)
    ctrlc::set_handler(move || {
        println!("\n'{}' was manually exited!!!", env!("CARGO_PKG_NAME"));
        process::exit(exitcode::SOFTWARE);
    })
    .map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "{} terminate signal invokation doesn't work!({:?})",
                env!("CARGO_PKG_NAME"),
                error
            ),
        )
    })?;

    // finished ipc sender reserved before collector takes action
    let ipc_send_finished = ipc_send.clone();

    // the collector
    // but yet this simple bracket to enclose this a little
    {
        trace!("syncing with 2 other threads");
        wait_collector.wait();
        trace!("sync with net and ui done ... collector can start");

        let synced_to_ui_messages = tx_from_collector_to_ui.clone();

        // set up data
        let collection_protected = SArc::new(SMutex::new(Collection::new()));

        // search parallelly
        let output_data = shared::collection_search(
            collection_protected.clone(),
            search_path,
            synced_to_ui_messages,
            has_ui,
        );

        info!("collector finished!!");
        if has_ui || has_net {
            // send own peer and others that are finished
            let memory_used = collection_protected
                .lock()
                .and_then(|collection| Ok(collection.memory()))
                .unwrap_or_else(|_| {
                    error!("locking collection didn't work here!");
                    0
                });

            let mut collection_output = IFCollectionOutputData::from(&output_data);
            // to have not 0 as data size minimum
            collection_output.size_of_data_in_kb = cmp::max(1, (memory_used / 1000) as usize);

            // to others via IPC
            if has_net {
                ipc_send_finished
                    .send(IPC::DoneSearching(collection_output.clone()))
                    .unwrap_or_else(|_| {
                        error!("net has to be up and receiving this send!");
                    });
            }
            // to myself (ui) via UiUpdateMsg
            if has_ui {
                tx_col
                    .send(UiUpdateMsg::PeerSearchFinished(
                        key_keeper::get_p2p_server_id(),
                        collection_output,
                    ))
                    .unwrap_or_else(|e| error!("use one: {}", e));
            }
        }
        if !has_ui {
            collection_protected
                .lock()
                .and_then(|locked_collection| {
                    Ok(locked_collection
                        .print_stats(&key_keeper::get_p2p_server_id(), nr_threads_for_collection))
                })
                .unwrap_or(())
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
