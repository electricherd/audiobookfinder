//! The oldest module, the data module stores all the data needed to collect.
use super::config;
use libp2p::PeerId;
use std::{
    cmp,                                     //max
    collections::hash_map::{Entry, HashMap}, // my main item uses a hash map
    fs::{self, DirEntry, Permissions},       // directory
    io,                                      // reading files
    os::linux::fs::MetadataExt,              //use std::os::windows::fs::MetadataExt;
    path::{Path, PathBuf},
}; // path, clear
use taglib;
use tree_magic;

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
    collection: HashMap<String, Box<InfoAlbum>>,
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
    audio: Audio,
    threads: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Audio {
    albums: u32,
    max_songs: usize,
}

type FileFn = dyn Fn(&mut Collection, &DirEntry, &mut FilesStat) -> io::Result<()>;

impl Collection {
    /// Sets up the whole collection that books all threads.
    pub fn new(peer_id: &PeerId, num_threads: usize) -> Collection {
        Collection {
            who: Worker::new(peer_id.clone(), num_threads),
            collection: HashMap::new(),
            stats: Stats {
                files: FilesStat {
                    analyzed: 0,
                    faulty: 0,
                    searched: 0,
                    other: 0,
                },
                audio: Audio {
                    albums: 0,
                    max_songs: 0,
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
        taglib::File::new(cb.to_str().unwrap())
            .and_then(|file| {
                match file.tag() {
                    Ok(tag) => {
                        let artist = tag.artist().unwrap_or("unknown".to_string());

                        self.stats.files.analyzed += 1;
                        file_stats.analyzed += 1;

                        let path_buffer = cb.to_path_buf();

                        let metadata = fs::metadata(cb).unwrap();
                        let filesize = metadata.st_size();
                        let permissions = metadata.permissions();

                        let possible_entry = FileInfo {
                            path: path_buffer,
                            size: filesize,
                            permissions: permissions,
                        };

                        match self.collection.entry(String::from(artist)) {
                            Entry::Occupied(mut entry) => {
                                entry.get_mut().reference_path.push(possible_entry);

                                let this_albums_length = entry.get().reference_path.len();
                                self.stats.audio.max_songs =
                                    cmp::max(self.stats.audio.max_songs, this_albums_length);
                            }
                            Entry::Vacant(entry) => {
                                let this_album: InfoAlbum = InfoAlbum {
                                    reference_path: vec![possible_entry],
                                };
                                entry.insert(Box::new(this_album));
                                self.stats.audio.albums += 1;
                            }
                        }
                    }
                    Err(_) => {
                        self.stats.files.faulty += 1;
                        file_stats.faulty += 1;
                    }
                }
                Ok(())
            })
            .or_else(|e| {
                error!("{:?}:{:?}", e, cb);
                Err(())
            })
    }

    pub fn print_stats(&self) {
        let output_string = format!(
            "This client's id     : {id:}\n\
             pathes/threads       : {nr_pathes:>width$}\n\
             albums found         : {albums_found:>width$}\n\
             most songs per album : {max_p_album:>width$}\n\
             ----------------------     \n\
             analyzed files       : {files_analyzed:>width$}\n\
             searched files       : {files_searched:>width$}\n\
             irrelevant files     : {files_irrelevant:>width$}\n\
             faulty files         : {files_faulty:>width$}\n",
            id = self.who.peer_id.to_string().to_uppercase(),
            nr_pathes = self.stats.threads,
            albums_found = self.stats.audio.albums,
            max_p_album = self.stats.audio.max_songs,
            files_analyzed = self.stats.files.analyzed,
            files_searched = self.stats.files.searched,
            files_irrelevant = self.stats.files.other,
            files_faulty = self.stats.files.faulty, // awesome, really
            width = 5
        );

        info!("{}", output_string);
    }
}
