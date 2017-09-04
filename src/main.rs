//#![feature(alloc_system)]
//extern crate alloc_system;   // strip down size of binary executable

extern crate id3;
extern crate tree_magic;  // mime types

use std::io;    
use std::env;     // args
use std::fs::{self, DirEntry};  // directory
use std::path::Path;  // path, clear
use std::collections::HashMap;


use id3::Tag;

/// generell info
struct InfoAlbum <'b> {
    reference_path : Vec<&'b Path>,
}

struct Collection<'b> {     
    collection : HashMap<&'b str, &'b InfoAlbum<'b> > 
}

type FileFn<'a,'b: 'a> = &'a Fn(&'a mut Collection, &'b DirEntry) -> io::Result<()>;

impl <'a,'b: 'a> Collection <'a> {

    pub fn new() -> Collection<'a>  {
        Collection { 
            collection : HashMap::new() 
        }
    }

    fn is_already_in_collection(&self, piece: &str) -> bool {
        self.collection.contains_key(piece)
    }

    /// The function that runs from the starting point
    fn visit_dirs(&mut self, dir: &'b Path, cb: FileFn<'a,'b>) -> io::Result<()> {
        if dir.is_dir() {
            let &mut test : &mut Collection<'a> = self;
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.visit_dirs(&path, cb)?;
                } else {
                    cb(test, &entry)?;
                }
            }
        }
        Ok(())
    }

    /// the function to check all files separately
    fn visit_files(col: &mut Collection, cb: &'b DirEntry) -> io::Result<()> {
        let filetype = tree_magic::from_filepath(&cb.path());
        match filetype.as_ref() {
            "text/plain" => {},
            "audio/mpeg" => col.visit_audio_files(&cb.path()),
            _ => println!("[{:?}]{:?}",filetype, cb.path()),
        }
    	Ok(())	
    }

    fn visit_audio_files(&mut self, cb: &'b Path) {
        let tag : Tag = Tag::read_from_path(cb).unwrap();
        println!("{:?}",tag.artist().unwrap())
    }
}


fn main() {
    let args: Vec<_> = env::args().collect();

    let mut collection : Collection<'static> = Collection::new();

    let mut path : &str = ".";
    if args.len() > 1 {
        path = &args[1];
    }
    let real_path = Path::new(path);
    match collection.visit_dirs(real_path, &Collection::visit_files) {
        _ => println!("Finished!")
    }
}
