//! A simple first program in rust, actually I plan to do something even
//! useful. Trying to find all my audiobooks on many machines, identify them,
//! find duplicates (later trying to solve the problem, also including their permissions, different names, but same albums, etc),
//! get all stats about it).

//#![feature(alloc_system)]
//extern crate alloc_system;   // strip down size of binary executable

extern crate clap;
extern crate hostname;
extern crate rayon;
extern crate uuid;   

use std::path::{Path};  // path, clear
use std::sync::{Arc, Mutex};
use std::thread;

use self::rayon::prelude::{IntoParallelRefIterator,
                           IndexedParallelIterator,
                           ParallelIterator};
use uuid::Uuid;

mod ctrl;
mod data;
mod net;

pub use self::data::Collection;
pub use self::ctrl::Ctrl;
pub use self::net::Net;

use ctrl::{SystemMsg,ReceiveDialog};
use ctrl::{Alive,Status};

static INPUT_FOLDERS  : &str = "folders";
static APP_TITLE      : &str = "The audiobook finder";
static ARG_NET        : &str = "net";
static ARG_TUI        : &str = "tui";

use std::sync::mpsc;

fn main() {
    let parse_args = clap::App::new(APP_TITLE)
                          .version("0.1")
                          .author("S. K. <skroemeke@gmail.com>")
                          .about("A audiobook finder")
                          .arg(clap::Arg::with_name("config")
                               .short("c")
                               .long("config")
                               .value_name("FILE")
                               .help("Sets a custom config file")
                               .takes_value(true))
                          .arg(clap::Arg::with_name(ARG_TUI)
                               .short("t")
                               .long("TUI")
                               .help("Starts the TUI")
                               .takes_value(false))
                          .arg(clap::Arg::with_name(ARG_NET)
                               .short("n")
                               .long("net")
                               .help("Starts without net")
                               .takes_value(false))                          
                          .arg(clap::Arg::with_name(INPUT_FOLDERS)
                               .help("Sets the input folder(s) to use")
                               .multiple(true)
                               .required(false))
                          .get_matches();

    // tricky thing, but I really like that
    let all_pathes = if let Some(correct_input) = parse_args.values_of(INPUT_FOLDERS) {
        correct_input.collect()
    } else {
        vec!(".")
    };

    let hostname = hostname::get_hostname().unwrap_or("undefined".to_string());
    let max_threads = rayon::current_num_threads();

    // check if tui and net search is needed 
    let has_arg = |x:&str| parse_args.is_present(x);

    let has_tui = has_arg(ARG_TUI);
    let has_net = has_arg(ARG_NET);



    // prepare the message system
    let (tx, rx) = mpsc::channel::<SystemMsg>();
    let tx_sys_mut  = Mutex::new(tx.clone());
    let tx_net_mut  = Mutex::new(tx.clone());

    // copy to vec<&str>
    let tui_pathes = all_pathes.iter().map(|s|s.to_string()).collect();

    // get an unique id for this client
    let client_id = Uuid::new_v4();

    // start the tui thread
    let tui_runner = thread::spawn(move || {
        if has_tui {
            let controller = Ctrl::new(client_id.to_string(),
                                       &tui_pathes,
                                       rx,
                                       tx.clone(),
                                       has_net);        
            match controller {
                Ok(mut controller) => {controller.run();},
                Err(_) => {}
            }
        }
    });

    // start the net runner thread
    let tx_net_mut_arc = Arc::new(tx_net_mut);          
    let net_runner = thread::spawn(move || {
      if has_net {
        // need to simplify and clarify this here ......
        let mut netfinder = Net::new(&client_id.to_string(),
                                     has_tui,
                                     tx_net_mut_arc.lock().unwrap().clone());
        netfinder.lookup();
      }
    });


    let init_collection = Collection::new(hostname, &client_id, max_threads);
    let collection_protected = Arc::new(Mutex::new(init_collection));

    // start the search threads, each path its own
    all_pathes.par_iter().enumerate().for_each(|(index,elem)| {
        if !has_tui {
          println!("[{:?}] looking into path {:?}", index, elem);
        } else {
            // start animation .... timer and so on
            tx_sys_mut.lock().unwrap().send(SystemMsg::StartAnimation(Alive::BUSYPATH(index),Status::ON)).unwrap();
        }
        let live_here = collection_protected.clone();
        let mut pure_collection = live_here.lock().unwrap();

        match pure_collection.visit_dirs(Path::new(elem),&data::Collection::visit_files) {
            Ok(local_stats) => {
                if has_tui {
                    // stop animation
                    tx_sys_mut.lock().unwrap().send(SystemMsg::StartAnimation(Alive::BUSYPATH(index),Status::OFF)).unwrap();
                    //let stat_message = ReceiveDialog::ShowNewPath{nr:index};
                    //tx_sys_mut.lock().unwrap().send(SystemMsg::Update(stat_message,text)).unwrap();
                } else {
                  let text = format!("\n\
                                    analyzed: {an:>width$}, faulty: {fa:>width$}\n\
                                    searched: {se:>width$}, other: {ot:>width$}",
                                    an=local_stats.analyzed, fa=local_stats.faulty,
                                    se=local_stats.searched, ot=local_stats.other,
                                    width=3);

                  println!("[{:?}] done {}",index,text);
                }
            },
            Err(_e) =>  {
                let text = format!("An error has occurred in search path [{}]!!", index);
                if has_tui {
                    let debug_message_id = ReceiveDialog::Debug;
                    let text = text.to_string();
                    tx_sys_mut.lock().unwrap().send(SystemMsg::Update(debug_message_id,text)).unwrap();
                } else {
                    println!("{:?}",text);
                }
            },
        }
    });

    if !has_tui {
        let result_collection = collection_protected.lock().unwrap();        
        result_collection.print_stats();
        let _ = net_runner.join();
    } else {
        let _ = tui_runner.join();
        // if tui, net runner shall stop when tui stops    
        drop(net_runner);
    }    
    
    println!("Finished!");
}
