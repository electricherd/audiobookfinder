//! The collection keeps and maintains all audio data.
use super::{super::config, bktree::BKTree};

use id3::Tag;
use libp2p_core::PeerId;
use std::{
    fs::{self, DirEntry}, // directory
    io,                   // reading files
    mem,
    path::Path,
    sync::{Arc as SArc, Mutex as SMutex},
};
use tree_magic;

static TOLERANCE: usize = 5;

struct AudioInfo {
    duration: u32,
    album: String,
    path: String,
    //computer: String,
}

// todo: think over this, peer and max threads ... kick it out
//       threads are interesting to fill in unused threads in path
//       for string distance with rayon!
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

pub struct Container {
    bk_tree: BKTree<String, Box<AudioInfo>>,
}
impl Container {
    pub fn new() -> Self {
        Self {
            bk_tree: BKTree::new(),
        }
    }
}

pub struct Collection {
    who: Worker,
    /// This collection contains all data
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
    memory: u64,
    files: FilesStat,
    threads: usize,
}

type FileFn =
    dyn Fn(&mut Collection, SArc<SMutex<Container>>, &DirEntry, &mut FilesStat) -> io::Result<()>;

impl Collection {
    /// Sets up the whole collection that books all threads.
    pub fn new(peer_id: &PeerId, num_threads: usize) -> Collection {
        Collection {
            who: Worker::new(peer_id.clone(), num_threads),
            stats: Stats {
                memory: 0,
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
    pub fn visit_path(
        &mut self,
        container: SArc<SMutex<Container>>,
        dir: &Path,
        cb: &FileFn,
    ) -> io::Result<FilesStat> {
        let mut file_stats = FilesStat {
            analyzed: 0,
            faulty: 0,
            searched: 0,
            other: 0,
        };

        if dir.is_dir() {
            // todo: go with free threads in the search with rayon
            for entry in fs::read_dir(dir)? {
                let mut loop_file_stats = &mut file_stats;

                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let file_stats_loop = self.visit_path(container.clone(), &path, cb)?;
                    loop_file_stats.add(&file_stats_loop);
                } else {
                    cb(self, container.clone(), &entry, &mut loop_file_stats).or_else(
                        |io_error| {
                            warn!("{:?}", io_error);
                            Err(io_error)
                        },
                    )?
                }
            }
        }
        Ok(file_stats)
    }

    /// the function to check all files separately
    pub fn visit_files(
        col: &mut Collection,
        data: SArc<SMutex<Container>>,
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
                        col.visit_audio_files(data, &cb.path(), file_stats)
                            .or_else(|_| {
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
    fn visit_audio_files(
        &mut self,
        data: SArc<SMutex<Container>>,
        cb: &Path,
        file_stats: &mut FilesStat,
    ) -> Result<(), ()> {
        Tag::read_from_path(cb.to_str().unwrap())
            .and_then(|tag| {
                let artist = tag.artist().unwrap_or("");
                let title = tag.title().unwrap_or("");
                let duration = tag.duration().unwrap_or(0);

                // audio book genre set is a strong indicator
                let _genre = tag.genre().unwrap_or("");                
                //
                let album = tag.album().unwrap_or("");
                let _album_artist = tag.album_artist().unwrap_or("");
                // many discs and total discs is a strong indicator
                let _disc = tag.disc().unwrap_or(0);
                let _total_discs = tag.total_discs().unwrap_or(0);
                //
                // having a good path pattern is a strong indicator: cb.to_str().unwrap()

                let _total_tracks = tag.total_tracks().unwrap_or(0);
                let _track = tag.track().unwrap_or(0);
                let _year = tag.year().unwrap_or(0);


                self.stats.files.analyzed += 1;
                file_stats.analyzed += 1;

                let mut has_enough_information = true;

                // artist + song name is key for bktree
                if artist.is_empty() && title.is_empty() {
                    has_enough_information = false;
                }

                if has_enough_information {
                    let key = [artist, title].join(" ");

                    // a) filter numbers / remove
                    // b) use album name / substract it?

                    // todo: bktree and levenshtein distance, use cosine similarity instead
                    let ref mut locked_container = data.lock().unwrap();
                    let (vec_exact_match, vec_similarities) = locked_container.bk_tree.find(&key, TOLERANCE);
                    if !vec_similarities.is_empty() {
                        trace!("close: {:?} to {:?},", &vec_similarities, &key);
                    }
                    // if exact match, don't insert!!
                    if vec_exact_match.is_empty() {
                        // todo: also decide when to not add and insert then

                        let key = key.clone();
                        let value = Box::new(AudioInfo {
                            duration,
                            album: album.to_string(),
                            path: cb.to_str().unwrap().to_string()
                        });
                        let mem_size = mem::size_of_val(&key) + mem::size_of_val(&value);
                        locked_container.bk_tree.insert(
                            key,
                            value,
                        );
                        self.stats.memory += mem_size as u64;
                    } else {
                        // exact match with certain AudioInfo
                        trace!(
                            "for {:?}, {} exact matches found!",
                            &key,
                            vec_exact_match.len()
                        );
                        for audio_info in vec_exact_match {
                            let time_distance = audio_info.duration as i32 - duration as i32;
                            if time_distance.abs() > 0 {
                                trace!(
                                    "same: but time differs {} seconds with album name old '{}' and new: '{}'!",
                                    time_distance,
                                    audio_info.album,
                                    album
                                );
                            }
                        }
                    }
                }
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
             memory               : {mem:>width$}kb\n\
             ----------------------     \n\
             analyzed files       : {files_analyzed:>width$}\n\
             searched files       : {files_searched:>width$}\n\
             irrelevant files     : {files_irrelevant:>width$}\n\
             faulty files         : {files_faulty:>width$}\n",
            id = self.who.peer_id.to_string().to_uppercase(),
            nr_pathes = self.stats.threads,
            mem = self.stats.memory / 1000,
            files_analyzed = self.stats.files.analyzed,
            files_searched = self.stats.files.searched,
            files_irrelevant = self.stats.files.other,
            files_faulty = self.stats.files.faulty, // awesome, really
            width = 5
        );

        info!("{}", output_string);
    }
}
