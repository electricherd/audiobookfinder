extern crate tree_magic;  // mime types

use std::io;    
use std::env;     // args
use std::fs::{self, DirEntry};  // directory
use std::path::Path;  // path, clear

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
    let you = match filetype.as_ref() {
        "text/plain" => "none",
        _ => "-",
    };
    println!("{:?}",you); //file_name());
	Ok(())	
}




fn main() {
    let args: Vec<_> = env::args().collect();

    let mut path = ".";
    if args.len() > 1 {
        path = &args[1];
    }
    let real_path = Path::new(path);
    visit_dirs(real_path, &visit_files);
}
