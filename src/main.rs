//#![feature(alloc_system)]
//extern crate alloc_system;   // strip down size of binary executable

extern crate id3;
extern crate tree_magic;  // mime types

use std::io;    
use std::env;     // args
use std::fs::{self, DirEntry};  // directory
use std::path::{Path,PathBuf};  // path, clear
use std::collections::HashMap;

use id3::Tag;

/// generell info
struct InfoAlbum {
    reference_path : Vec<PathBuf>
}

struct Collection {     
    collection : HashMap<String, Box<InfoAlbum>>
}

type FileFn = Fn(&mut Collection, &DirEntry) -> io::Result<()>;

impl Collection {

    pub fn new() -> Collection {
        Collection { 
            collection : HashMap::new() 
        }
    }

    fn is_already_in_collection(&self, piece: &str) -> bool {
        self.collection.contains_key(piece)
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
        let filetype = tree_magic::from_filepath(&cb.path());
        match filetype.as_ref() {
            "text/plain" => {},
            "audio/mpeg" => col.visit_audio_files(&cb.path()),
            _ => println!("[{:?}]{:?}",filetype, cb.path()),
        }
    	Ok(())	
    }

    fn visit_audio_files(&mut self, cb: &Path) {
        let tag : Tag = Tag::read_from_path(cb).unwrap();
        let artist = tag.artist().unwrap();
        if self.is_already_in_collection(artist) {
            if let Some(found_box) = self.collection.get_mut(artist) {
                print!("{{");
                for i in &found_box.reference_path {
                    print!("{:?}, ",i);
                }
                println!("}}");                
                found_box.reference_path.push(cb.to_path_buf());
            } else {
                //let this_album : InfoAlbum = InfoAlbum { reference_path: vec!(cb.to_path_buf()) };
                //self.collection.insert(artist.to_owned(),Box::new(this_album));
                //println!("*****{:?}",artist);
            }
        } else {
            let this_album : InfoAlbum = InfoAlbum { reference_path: vec!(cb.to_path_buf()) };
            self.collection.insert(artist.to_owned(),Box::new(this_album));
            println!("*****{:?}",artist);
        }
    }
}


fn main() {
    let args: Vec<_> = env::args().collect();

    let mut collection = Collection::new();

    let mut path = ".";
    if args.len() > 1 {
        path = &args[1];
    }
    let real_path = Path::new(path);
    match collection.visit_dirs(real_path, &Collection::visit_files) {
        _ => println!("Finished!")
    }
}
