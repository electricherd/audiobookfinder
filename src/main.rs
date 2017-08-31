//#![feature(alloc_system)]
//extern crate alloc_system;   // strip down size of binary executable

extern crate id3;
extern crate tree_magic;  // mime types

use std::io;    
use std::env;     // args
use std::fs::{self, DirEntry};  // directory
use std::path::Path;  // path, clear

use id3::Tag;

fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry) -> io::Result<()> ) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

fn visit_files(cb: &DirEntry) -> io::Result<()> {
    let filetype = tree_magic::from_filepath(&cb.path());
    match filetype.as_ref() {
        "text/plain" => {},
        "audio/mpeg" => visit_audio_files(&cb.path()),
        _ => println!("[{:?}]{:?}",filetype, cb.path()),
    }
	Ok(())	
}

fn visit_audio_files(cb: &Path) {
    let tag = Tag::read_from_path(cb).unwrap();
    println!("{:?}",tag.artist().unwrap())
}



fn main() {
    let args: Vec<_> = env::args().collect();

    let mut path = ".";
    if args.len() > 1 {
        path = &args[1];
    }
    let real_path = Path::new(path);
    match visit_dirs(real_path, &visit_files) {
        _ => println!("Finished!")
    }
}
