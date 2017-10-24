//! A simple first program in rust, actually I plan to do something even
//! useful. Trying to find all my audiobooks on many machines, identify them,
//! find duplicates (later trying to solve the problem, also including their permissions, different names, but same albums, etc),
//! get all stats about it).

//#![feature(alloc_system)]
//extern crate alloc_system;   // strip down size of binary executable

//extern crate hyper;   // sometime, for a good server / client over https communication
extern crate clap;
extern crate id3;
extern crate tree_magic;  // mime types
extern crate rayon;
extern crate hostname;

mod ctrl;

pub use self::ctrl::Ctrl;

use id3::Tag;     // to identify the audio files

use std::io;      // reading files 

use std::cmp;     // max
use std::fs::{self, DirEntry, Permissions};  // directory
use std::path::{Path,PathBuf};  // path, clear
use std::collections::hash_map::{HashMap,Entry}; // my main item uses a hash map
use std::sync::{Arc, Mutex};                     // safe containment and locking
//use std::os::windows::fs::MetadataExt;
use std::os::linux::fs::MetadataExt;
use std::thread;


use rayon::prelude::*;                           // threading with iterators


#[allow(dead_code)]
/// # File info
/// All info on a audio files
/// (let's see how info we need,
/// its size vs necessary info)
struct FileInfo {
    path : PathBuf,
    size : u64,
    permissions: Permissions
}


/// # Album information
/// General info
struct InfoAlbum {
    reference_path : Vec<FileInfo>
}

#[allow(dead_code)]
struct Worker {
    /// identify them.
    id       : u32, 
    hostname : String,
    max_threads : usize,
}

/// # Worker
/// the worker is supposed to be running on different machines
/// with one server and many clients
impl Worker {
    /// # Just new
    /// To identify how to comment
    /// # Arguments
    /// * '_hostname' - The hostname from remote/local
    /// * '_id' - the identification (each will create an own hash)
    /// * '_maxthreads' - how many threads can the worker create
    pub fn new(_hostname: String, _id: u32, _maxthreads : usize) -> Worker {
        Worker { hostname: _hostname, id : _id, max_threads: _maxthreads}
    }
}


struct Collection {    
    /// This collection contains all data
    who         : Worker,
    collection  : HashMap<String, Box<InfoAlbum>>,
    stats       : Stats
}

struct Stats {
    files : Files,
    audio : Audio,
    threads : u8,
}

struct Audio {
    albums    : u32,
    max_songs : usize,
}

struct Files {
    analyzed: u32,
    faulty :  u32,
    searched: u32,
    other:    u32,
}


type FileFn = Fn(&mut Collection, &DirEntry) -> io::Result<()>;

/// This part implements all functions
impl Collection {
    pub fn new(_hostname: String, _numthreads: usize) -> Collection {
        Collection { 
            who   :  Worker::new(_hostname, 0, _numthreads),
            collection : HashMap::new(), 
            stats      : Stats { 
                          files: Files {analyzed: 0, faulty: 0, searched: 0, other: 0},
                          audio: Audio {albums: 0, max_songs: 0},
                          threads: 0
                        }
        }
    }

    /// The function that runs from the starting point
    fn visit_dirs(&mut self, dir: &Path, cb: &FileFn) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.visit_dirs(&path, cb)?;
                } else {
                    cb(self, &entry)?;
                }
            }
        }
        Ok(())
    }

    /// the function to check all files separately
    fn visit_files(col: &mut Collection, cb: &DirEntry) -> io::Result<()> {
        // count stats
        col.stats.files.searched += 1;        
        
        let filetype = tree_magic::from_filepath(&cb.path());
        let prefix = filetype.split("/").nth(0);
        match prefix {
            Some("audio") => col.visit_audio_files(&cb.path())?,
            Some("text") | Some("application") 
             | Some("image") => {},
            _ => {
                 println!("[{:?}]{:?}",prefix, cb.path());
                 col.stats.files.other += 1;
            }
        }
    	Ok(())	
    }

    fn visit_audio_files(&mut self, cb: &Path) -> io::Result<()> {
        match Tag::read_from_path(cb) {
            Ok(tag) => {
                if let Some(artist) = tag.artist() {
                    self.stats.files.analyzed += 1;                    

                    let path_buffer = cb.to_path_buf();

                    let metadata = fs::metadata(cb)?;
                    let filesize = metadata.st_size();
                    let permissions = metadata.permissions();

                    let possible_entry = FileInfo{path: path_buffer, size : filesize, permissions: permissions};

                    match self.collection.entry(String::from(artist)) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().reference_path.push(possible_entry);

                            let this_albums_length = entry.get().reference_path.len();
                            self.stats.audio.max_songs = cmp::max(self.stats.audio.max_songs,this_albums_length);
                        }
                        Entry::Vacant(entry) => {
                            let this_album : InfoAlbum = InfoAlbum { reference_path: vec!(possible_entry) };
                            entry.insert(Box::new(this_album));
                            self.stats.audio.albums += 1;
                        }
                    }
                } else {
                    self.stats.files.faulty += 1;
                }
            },
            Err(_) => { self.stats.files.faulty += 1}
        }
        Ok(())
   }

   pub fn print_stats(&self) {
        println!("pathes/threads       : {:?}", self.stats.threads);     
        println!("albums found         : {:?}", self.stats.audio.albums); 
        println!("most songs per album : {:?}", self.stats.audio.max_songs); 
        println!("----------------------");
        println!("analyzed files       : {:?}", self.stats.files.analyzed);    
        println!("searched files       : {:?}", self.stats.files.searched);    
        println!("irrelevant files     : {:?}", self.stats.files.other);    
        println!("faulty files         : {:?}", self.stats.files.faulty);
   }
} // end of impl Collection


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

    let data_runner = thread::spawn(|| {
        let controller = Ctrl::new(&all_pathes);
    });


    all_pathes.par_iter().for_each(|elem| {
        //rayon::current_thread_index()
        println!("Start path {:?}", elem);
        let live_here = collection_protected.clone();

        let mut pure_collection = live_here.lock().unwrap();
        let _ = pure_collection.visit_dirs(Path::new(elem),&Collection::visit_files);
    });

    let result_collection = collection_protected.lock().unwrap();
    result_collection.print_stats();

    data_runner.join();
    
    println!("Finished!");
}
