//#![feature(alloc_system)]
//extern crate alloc_system;   // strip down size of binary executable

extern crate id3;
extern crate tree_magic;  // mime types

use std::io;    
use std::env;     // args
use std::fs::{self, DirEntry};  // directory
use std::path::Path;  // path, clear
use std::iter::*;

use id3::Tag;

/// generell info
struct InfoAlbum <'b> {
    album_name : &'b str,
    reference_path : Vec<&'b Path>,
}

struct Collection<'b> { collection : Vec<&'b InfoAlbum<'b> > }

impl <'b> Collection <'b> {

    pub fn new() -> Collection<'b>  {
        Collection { 
            collection : Vec::new() 
        }
    }

    fn is_already_in_collection(&self, piece: &str) -> bool {
        //self.collection.iter().position(|&&album_name| album_name == piece) //find(album_name == piece)
        true
    }

    /// The function that runs from the starting point
    fn visit_dirs(&self, dir: &Path, cb: &Fn(&DirEntry) -> io::Result<()> ) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.visit_dirs(&path, cb)?;
                } else {
                    cb(&entry)?;
                }
            }
        }
        Ok(())
    }

    /// the function to check all files separately
    fn visit_files(cb: &DirEntry) -> io::Result<()> {
        let filetype = tree_magic::from_filepath(&cb.path());
        match filetype.as_ref() {
            "text/plain" => {},
            "audio/mpeg" => Collection::visit_audio_files(&cb.path()),
            _ => println!("[{:?}]{:?}",filetype, cb.path()),
        }
    	Ok(())	
    }

    fn visit_audio_files(cb: &Path) {
        let tag : Tag = Tag::read_from_path(cb).unwrap();

        println!("{:?}",tag.artist().unwrap())
    }
}


fn main() {
    let args: Vec<_> = env::args().collect();

    let collection : Collection<'static> = Collection::new();


    let mut path : &str = ".";
    if args.len() > 1 {
        path = &args[1];
    }
    let real_path = Path::new(path);
    match collection.visit_dirs(real_path, &Collection::visit_files) {
        _ => println!("Finished!")
    }
}
