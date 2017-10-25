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
use std::sync::{Arc, Mutex};                     // safe containment and locking
use std::thread;
use self::rayon::prelude::*;                           // threading with iterators

mod ctrl;
mod data;
pub use self::data::Collection;
pub use self::ctrl::Ctrl;


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

    //
    let max_threads = rayon::current_num_threads();

    let init_collection = Collection::new(hostname, max_threads);
    let collection_protected = Arc::new(Mutex::new(init_collection));


    // f*ck in order to copy vec<&str>
    let tui_pathes = all_pathes.iter().map(|s|s.to_string()).collect();

    // 
    let tui_runner = thread::spawn(move || {
        let controller = Ctrl::new(&tui_pathes);
        match controller {
            Ok(mut controller) => {controller.run();},
            Err(_) => {}
        }
    });

    all_pathes.par_iter().for_each(|elem| {
        //rayon::current_thread_index()
        println!("Start path {:?}", elem);
        let live_here = collection_protected.clone();

        let mut pure_collection = live_here.lock().unwrap();
        let _ = pure_collection.visit_dirs(Path::new(elem),&data::Collection::visit_files);
    });

    let result_collection = collection_protected.lock().unwrap();
    result_collection.print_stats();

    let _ = tui_runner.join();
    
    println!("Finished!");
}
