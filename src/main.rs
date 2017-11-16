//! A simple first program in rust, actually I plan to do something even
//! useful. Trying to find all my audiobooks on many machines, identify them,
//! find duplicates (later trying to solve the problem, also including their permissions, different names, but same albums, etc),
//! get all stats about it).

//#![feature(alloc_system)]
//extern crate alloc_system;   // strip down size of binary executable

extern crate clap;
extern crate hostname;
extern crate rayon;

use std::path::{Path};  // path, clear
use std::sync::{Arc, Mutex};
use std::thread;
use self::rayon::prelude::{IntoParallelRefIterator,
                           IndexedParallelIterator,
                           ParallelIterator};

mod ctrl;
mod data;
pub use self::data::Collection;
pub use self::ctrl::Ctrl;

use ctrl::{SystemMsg,ReceiveDialog};

static INPUT_FOLDERS : &str = "folders";
static APP_TITLE : &str = "The audiobook finder";
static ARG_TUI : &str = "tui";

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

    // check if tui is needed 
    let has_tui = parse_args.is_present(ARG_TUI);


    // prepare the message system
    let (tx, rx) = mpsc::channel::<SystemMsg>();
    let tx_mut = Mutex::new(tx.clone());


    // f*ck in order to copy vec<&str>
    let tui_pathes = all_pathes.iter().map(|s|s.to_string()).collect();

    // start the tui thread
    let tui_runner = thread::spawn(move || {
        if has_tui {
            let controller = Ctrl::new(&tui_pathes,rx,tx);        
            match controller {
                Ok(mut controller) => {controller.run();},
                Err(_) => {}
            }
        }
    });


    let init_collection = Collection::new(hostname, max_threads);
    let collection_protected = Arc::new(Mutex::new(init_collection));

    // start the search threads, each path its own
    all_pathes.par_iter().enumerate().for_each(|(index,elem)| {
        if !has_tui {
            println!("[{:?}] looking into path {:?}", index, elem);
        }
        let live_here = collection_protected.clone();

        let mut pure_collection = live_here.lock().unwrap();

        if let Err(_e) = pure_collection.visit_dirs(Path::new(elem),&data::Collection::visit_files) {
            let text = format!("An error has occurred in search path [{}]!!", index);
            if has_tui {
                let debug_message_id = ReceiveDialog::Debug;
                let text = text.to_string();
                tx_mut.lock().unwrap().send(SystemMsg::Update(debug_message_id,text)).unwrap();
            } else {
                println!("{:?}",text);
            }
        } else {
            // all good, so write some (yet debug) text
            if has_tui {
                let text = format!("test{}",index);
                let path_index = ReceiveDialog::PathNr{nr:index};
                tx_mut.lock().unwrap().send(SystemMsg::Update(path_index, text)).unwrap();
            }
        }
    });

    if !has_tui {
        let result_collection = collection_protected.lock().unwrap();        
        result_collection.print_stats();
    }
    let _ = tui_runner.join();
    
    println!("Finished!");
}
