//! The oldest module, the data module stores all the data needed to collect.
pub mod bktree;
pub mod ipc;

use super::config;
use bktree::BKTree;

use id3::Tag;
use libp2p_core::PeerId;
use std::{
    fs::{self, DirEntry, Permissions}, // directory
    io,                                // reading files
    path::{Path, PathBuf},
    sync::Arc,
};
use tree_magic;

static TOLERANCE: usize = 5;
static TIME_TOLERANCE_SEC: usize = 4;

#[allow(dead_code)]
/// # File info
/// All info on a audio files
/// (let's see how info we need,
/// its size vs necessary info)
struct FileInfo {
    path: PathBuf,
    size: u64,
    permissions: Permissions,
}

struct AudioInfo {
    duration: u32,
    album: String,
}

/// # Album information
/// General info
struct InfoAlbum {
    reference_path: Vec<FileInfo>,
}

#[allow(dead_code)]
struct Worker {
    /// identify them.
    peer_id: PeerId,
    max_threads: usize,
}

/// # Worker
/// the worker is supposed to be running on different machines
/// with one server and many clients
impl Worker {
    /// # Just new
    /// To identify how to comment
    /// # Arguments
    /// * 'id' - the identification (each will create an own hash)
    /// * 'max_threads' - how many threads can the worker create
    pub fn new(peer_id: PeerId, max_threads: usize) -> Worker {
        Worker {
            peer_id: peer_id,
            max_threads: max_threads,
        }
    }
}

pub struct Collection {
    /// This collection contains all data
    who: Worker,
    bk_tree: BKTree<String, Box<AudioInfo>>,
    stats: Stats,
}
/// Only some statistics
pub struct FilesStat {
    pub analyzed: u32,
    pub faulty: u32,
    pub searched: u32,
    pub other: u32,
}

impl FilesStat {
    /// Adds stats from one to the other,
    /// used for combining different thread
    /// results.
    fn add(&mut self, other: &FilesStat) {
        self.analyzed += other.analyzed;
        self.faulty += other.faulty;
        self.searched += other.searched;
        self.other += other.other;
    }
}

struct Stats {
    files: FilesStat,
    threads: usize,
}

type FileFn = dyn Fn(&mut Collection, &DirEntry, &mut FilesStat) -> io::Result<()>;

impl Collection {
    /// Sets up the whole collection that books all threads.
    pub fn new(peer_id: &PeerId, num_threads: usize) -> Collection {
        Collection {
            who: Worker::new(peer_id.clone(), num_threads),
            bk_tree: BKTree::new(),
            stats: Stats {
                files: FilesStat {
                    analyzed: 0,
                    faulty: 0,
                    searched: 0,
                    other: 0,
                },
                threads: num_threads,
            },
        }
    }

    /// The function that runs from a certain path
    pub fn visit_path(&mut self, dir: &Path, cb: &FileFn) -> io::Result<FilesStat> {
        let mut file_stats = FilesStat {
            analyzed: 0,
            faulty: 0,
            searched: 0,
            other: 0,
        };

        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let mut loop_file_stats = &mut file_stats;

                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let file_stats_loop = self.visit_path(&path, cb)?;
                    loop_file_stats.add(&file_stats_loop);
                } else {
                    cb(self, &entry, &mut loop_file_stats).or_else(|io_error| {
                        warn!("{:?}", io_error);
                        Err(io_error)
                    })?
                }
            }
        }
        Ok(file_stats)
    }

    /// the function to check all files separately
    pub fn visit_files(
        col: &mut Collection,
        cb: &DirEntry,
        file_stats: &mut FilesStat,
    ) -> io::Result<()> {
        // count stats
        col.stats.files.searched += 1;
        file_stats.searched += 1;

        let filetype = tree_magic::from_filepath(&cb.path());
        let prefix = filetype.split("/").nth(0);
        match prefix {
            Some("audio") => {
                if let Some(suffix) = filetype.split("/").last() {
                    if config::data::IGNORE_AUDIO_FORMATS
                        .iter()
                        .any(|&s| s == suffix)
                    {
                        col.stats.files.other += 1;
                        file_stats.other += 1;
                        Ok(())
                    } else {
                        col.visit_audio_files(&cb.path(), file_stats).or_else(|_| {
                            col.stats.files.faulty += 1;
                            file_stats.faulty += 1;
                            error!("ts: {:?}", filetype);
                            Err(io::Error::new(io::ErrorKind::Other, "unknown audio file!"))
                        })
                    }
                } else {
                    col.stats.files.faulty += 1;
                    file_stats.faulty += 1;
                    Ok(())
                }
            }
            // FIXME: video in taglib holds also oga which are audio indeed ...
            Some("text") | Some("application") | Some("image") | Some("video") => Ok(()),
            _ => {
                error!("[{:?}]{:?}", prefix, cb.path());
                col.stats.files.other += 1;
                file_stats.other += 1;
                Ok(())
            }
        }
    }

    /// Check the file and retrieve the meta-data info
    fn visit_audio_files(&mut self, cb: &Path, file_stats: &mut FilesStat) -> Result<(), ()> {
        Tag::read_from_path(cb.to_str().unwrap())
            .and_then(|tag| {
                let artist = tag.artist().unwrap_or("");
                let title = tag.title().unwrap_or("");
                let duration = tag.duration().unwrap_or(0);
                let album = tag.album().unwrap_or("");

                self.stats.files.analyzed += 1;
                file_stats.analyzed += 1;

                let path_buffer = cb.to_path_buf();

                let metadata = fs::metadata(cb).unwrap();
                let filesize = metadata.len();
                let permissions = metadata.permissions();

                let possible_entry = FileInfo {
                    path: path_buffer,
                    size: filesize,
                    permissions: permissions,
                };

                // artist + song name is key for bktree
                let key = [artist, title].join(" ");

                let (vec_k, vec_c) = self.bk_tree.find(&key, TOLERANCE);
                if !vec_c.is_empty() {
                    let mut vec_similar = vec![];
                    for (index, similar) in vec_k.iter().enumerate() {
                        let time_distance = similar.duration as i32 - duration as i32;
                        if time_distance.abs() < TIME_TOLERANCE_SEC as i32 {
                            vec_similar.push(vec_c[index]);
                        }
                    }
                    if !vec_similar.is_empty() {
                        trace!(
                            "close: {:?} to {:?}, {} similar",
                            &vec_similar,
                            &key,
                            &vec_k.len()
                        );
                    }
                }

                // todo: decide when to not add and insert then
                self.bk_tree.insert(
                    key.clone(),
                    Box::new(AudioInfo {
                        duration,
                        album: album.to_string(),
                    }),
                );
                Ok(())
            })
            .or_else(|_| {
                self.stats.files.faulty += 1;
                file_stats.faulty += 1;
                Ok(())
            })
    }

    pub fn print_stats(&self) {
        let output_string = format!(
            "This client's id     : {id:}\n\
             pathes/threads       : {nr_pathes:>width$}\n\
             ----------------------     \n\
             analyzed files       : {files_analyzed:>width$}\n\
             searched files       : {files_searched:>width$}\n\
             irrelevant files     : {files_irrelevant:>width$}\n\
             faulty files         : {files_faulty:>width$}\n",
            id = self.who.peer_id.to_string().to_uppercase(),
            nr_pathes = self.stats.threads,
            files_analyzed = self.stats.files.analyzed,
            files_searched = self.stats.files.searched,
            files_irrelevant = self.stats.files.other,
            files_faulty = self.stats.files.faulty, // awesome, really
            width = 5
        );

        info!("{}", output_string);
    }
}
