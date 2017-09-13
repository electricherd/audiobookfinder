//#![feature(alloc_system)]
//extern crate alloc_system;   // strip down size of binary executable

extern crate id3;
extern crate tree_magic;  // mime types
extern crate crossbeam;

use id3::Tag;

use std::io;
use std::env;     // args
use std::cmp;     // max
use std::fs::{self, DirEntry};  // directory
use std::path::{Path,PathBuf};  // path, clear
use std::collections::hash_map::{HashMap,Entry};

/// general info
struct InfoAlbum {
    reference_path : Vec<PathBuf>
}

struct Collection {     
    collection  : HashMap<String, Box<InfoAlbum>>,
    stats       : Stats
}

struct Stats {
    files : Files,
    audio : Audio,
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

impl Collection {

    pub fn new() -> Collection {
        Collection { 
            collection : HashMap::new(), 
            stats      : Stats { 
                          files: Files {analyzed: 0, faulty: 0, searched: 0, other: 0},
                          audio: Audio {albums: 0, max_songs: 0},
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
            Some("audio") => col.visit_audio_files(&cb.path()),
            Some("text") | Some("application") 
             | Some("image") => {},
            _ => {
                 println!("[{:?}]{:?}",prefix, cb.path());
                 col.stats.files.other += 1;
            }
        }
    	Ok(())	
    }

    fn visit_audio_files(&mut self, cb: &Path) {
        match Tag::read_from_path(cb) {
            Ok(tag) => {
                if let Some(artist) = tag.artist() {
                    self.stats.files.analyzed += 1;                    

                    let path_buffer = cb.to_path_buf();
                    match self.collection.entry(String::from(artist)) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().reference_path.push(path_buffer);

                            let this_albums_length = entry.get().reference_path.len();
                            self.stats.audio.max_songs = cmp::max(self.stats.audio.max_songs,this_albums_length)
                        }
                        Entry::Vacant(entry) => {
                            let this_album : InfoAlbum = InfoAlbum { reference_path: vec!(path_buffer) };
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
   }

   pub fn print_stats(&self) {
        println!("albums found         : {:?}", self.stats.audio.albums); 
        println!("most songs per album : {:?}", self.stats.audio.max_songs); 
        println!("----------------------");
        println!("analyzed files       : {:?}", self.stats.files.analyzed);    
        println!("searched files       : {:?}", self.stats.files.searched);    
        println!("irrelevant files     : {:?}", self.stats.files.other);    
        println!("faulty files         : {:?}", self.stats.files.faulty);
   }
} // end of impl Collection


fn runner(path: &str) {
    let mut collection = Collection::new();    

    crossbeam::scope(|scope| {
        scope.spawn(|| {        
            collection.visit_dirs(Path::new(path), &Collection::visit_files);
            collection.print_stats();
        })
    });
}


fn main() {
    let args : Vec<_> = env::args().collect();

    let mut path = ".";
    if args.len() > 1 {
        path = &args[1];
    }

    runner(path);
//        _ => println!("Done!")
//    }
    println!("Finished!");
}
