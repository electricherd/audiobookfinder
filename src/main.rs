use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

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
	println!("{:?}",cb.path()); //file_name());
	Ok(())	
}




fn main() {
    let path = Path::new(".");
    visit_dirs(path, &visit_files);
}
