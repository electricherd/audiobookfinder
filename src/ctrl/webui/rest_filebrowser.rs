///! A helper to make local computer browsing of file paths possible
///! on smarter devices a dialog would do it ....
use dirs;
use std::{
    fs::{self, DirEntry},
    path::Path,
    string::String,
    vec::Vec,
};

pub fn return_directory(given: String) -> Vec<String> {
    let trying_path = if given.is_empty() {
        dirs::home_dir()
    } else {
        Path::new(&given).canonicalize().ok()
    };

    match trying_path {
        Some(try_as_dir) => {
            if let Ok(good_dir) = fs::read_dir(&try_as_dir) {
                let mut return_vec = vec![];
                for entry in good_dir {
                    if entry.is_err() {
                        continue;
                    }
                    let entry = entry.unwrap().path();
                    if entry.is_dir() {
                        if let Some(unicode_str) = entry.to_str() {
                            return_vec.push(unicode_str.to_string());
                        }
                    }
                }
                return_vec
            } else {
                error!("Path '{:?}' was no dir!", &try_as_dir);
                vec![]
            }
        }
        None => {
            error!("Path '{:?}' was not good!", given);
            vec![]
        }
    }
}
