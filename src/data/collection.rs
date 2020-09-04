//! The collection keeps and maintains all audio data.
use super::{
    super::common::config,
    audio_info::{AudioInfo, Container},
    ipc::IPC,
    tag_readers::{
        CommonAudioInfo, FlacTagReader, ID3TagReader, MP3TagReader, MP4TagReader, TagReader,
    },
};
use crate::data::audio_info::AudioInfoKey;
use crossbeam::channel::Sender as CrossbeamSender;
use libp2p_core::PeerId;
use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    io::{self, BufReader},
    mem,
    path::Path,
    sync::{Arc as SArc, Mutex as SMutex},
    time::Duration,
};
use tree_magic_mini;

static TOLERANCE: usize = 5;
/// distance of Levenshtein-algorithm
static ID3_CAPACITY: usize = 1024;
/// capacity to read small portion of file

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

/// Collection keeps control and result data of the build-up of the container.
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
    pub duplicates: u32,
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

type FileFn = dyn Fn(
    &mut Collection,
    SArc<SMutex<Container>>,
    &DirEntry,
    CrossbeamSender<IPC>,
    &mut FilesStat,
) -> io::Result<()>;

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
                    duplicates: 0,
                },
                threads: num_threads,
            },
        }
    }

    /// The function that runs from a given path
    pub fn visit_path(
        &mut self,
        container: SArc<SMutex<Container>>,
        dir: &Path,
        cb: &FileFn,
        ipc_sender: CrossbeamSender<IPC>,
    ) -> io::Result<FilesStat> {
        let mut file_stats = FilesStat {
            analyzed: 0,
            faulty: 0,
            searched: 0,
            other: 0,
            duplicates: 0,
        };

        if dir.is_dir() {
            // todo: go with free threads in the search with rayon
            for entry in fs::read_dir(dir)? {
                let ipc_loop_sender = ipc_sender.clone();
                let mut loop_file_stats = &mut file_stats;

                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let file_stats_loop =
                        self.visit_path(container.clone(), &path, cb, ipc_loop_sender)?;
                    loop_file_stats.add(&file_stats_loop);
                } else {
                    cb(
                        self,
                        container.clone(),
                        &entry,
                        ipc_loop_sender,
                        &mut loop_file_stats,
                    )
                    .or_else(|io_error| {
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
        data: SArc<SMutex<Container>>,
        cb: &DirEntry,
        ipc_sender: CrossbeamSender<IPC>,
        file_stats: &mut FilesStat,
    ) -> io::Result<()> {
        // count stats
        col.stats.files.searched += 1;
        file_stats.searched += 1;

        if let Some(mime_type) = tree_magic_mini::from_filepath(&cb.path()) {
            let vec_type: Vec<&str> = mime_type.split("/").collect();
            if vec_type.len() == 2 {
                let (prefix, suffix) = (vec_type[0], vec_type[1]);
                match prefix {
                    // some audio files have video-mimetype
                    "audio" | "video" => {
                        if config::data::IGNORE_AUDIO_FORMATS
                            .iter()
                            .any(|&s| s == suffix)
                        {
                            col.stats.files.other += 1;
                            file_stats.other += 1;
                            Ok(())
                        } else {
                            col.visit_audio_files(data, suffix, &cb.path(), ipc_sender, file_stats)
                                .or_else(|_| {
                                    col.stats.files.faulty += 1;
                                    file_stats.faulty += 1;
                                    error!("ts: {:?}", mime_type);
                                    Err(io::Error::new(io::ErrorKind::Other, "unknown audio file!"))
                                })
                        }
                    }
                    "text" | "application" | "image" => Ok(()),
                    _ => {
                        error!("[{:?}]{:?}", prefix, cb.path());
                        col.stats.files.other += 1;
                        file_stats.other += 1;
                        Ok(())
                    }
                }
            } else {
                col.stats.files.faulty += 1;
                file_stats.faulty += 1;
                Ok(())
            }
        } else {
            // not readable mime-type is no error
            Ok(())
        }
    }

    /// Check the file and retrieve the meta-data info
    fn visit_audio_files<'a>(
        &mut self,
        data: SArc<SMutex<Container>>,
        suffix: &'a str,
        cb: &Path,
        ipc_sender: CrossbeamSender<IPC>,
        file_stats: &mut FilesStat,
    ) -> Result<(), ()> {
        // open file only once
        // fixme: fix unwraps here
        let file_name = cb.to_str().unwrap();
        let file = std::fs::File::open(file_name).unwrap();
        let mut file_buffer = BufReader::with_capacity(ID3_CAPACITY, file);

        let mut processed = false;
        let mut all_known_suffixes = HashSet::<&str>::new();

        // cosy little helper (capturing suffix)
        let suffix_has = |v: Vec<&str>| v.iter().any(|&s| s == suffix);

        // 2nd cosy helper
        let mut analyze = |tag_reader: &Box<dyn TagReader<'static> + Sync>, ipc_loop_sender| {
            if !processed {
                if suffix_has(tag_reader.known_suffixes()) {
                    if let Ok(tag_data) = tag_reader.read_tag_from(&mut file_buffer) {
                        self.analyze_tag(
                            data.clone(),
                            ipc_loop_sender,
                            file_stats,
                            file_name.to_string(),
                            &tag_data,
                        );
                        processed = true;
                    }
                }
            }
        };

        // only to be done once
        lazy_static! {
            static ref ANALYZE_ORDER: [Box<dyn TagReader<'static> + Sync>; 4] = [
                Box::new(MP4TagReader),
                Box::new(FlacTagReader),
                Box::new(ID3TagReader),
                Box::new(MP3TagReader),
            ];
        }
        // analyze according to order
        for reader in ANALYZE_ORDER.iter().to_owned() {
            analyze(reader, ipc_sender.clone());
            reader
                .known_suffixes()
                .iter_mut()
                .all(|mime_suffix| all_known_suffixes.insert(*mime_suffix));
        }

        if !processed {
            if all_known_suffixes.contains(suffix) {
                warn!(
                    "though known, could not process mime-type suffix: {} - path: {}!",
                    suffix, file_name
                );
            } else {
                warn!("this suffix {} has no tag reader yet!", suffix);
            }
            self.stats.files.faulty += 1;
            file_stats.faulty += 1;
        }
        Ok(())
    }

    pub fn print_stats(&self) {
        let output_string = format!(
            "This client's id     : {id:}\n\
             paths/threads        : {nr_pathes:>width$}\n\
             memory               : {mem:>width$}kb\n\
             ----------------------     \n\
             analyzed files       : {files_analyzed:>width$}\n\
             searched files       : {files_searched:>width$}\n\
             duplicate files      : {files_duplicate:>width$}\n\
             irrelevant files     : {files_irrelevant:>width$}\n\
             faulty files         : {files_faulty:>width$}\n",
            id = self.who.peer_id.to_string().to_uppercase(),
            nr_pathes = self.stats.threads,
            mem = self.stats.memory / 1000,
            files_analyzed = self.stats.files.analyzed,
            files_searched = self.stats.files.searched,
            files_duplicate = self.stats.files.duplicates,
            files_irrelevant = self.stats.files.other,
            files_faulty = self.stats.files.faulty, // awesome, really
            width = 5
        );

        info!("{}", output_string);
    }

    fn analyze_tag<'a>(
        &mut self,
        data: SArc<SMutex<Container>>,
        ipc_sender: CrossbeamSender<IPC>,
        file_stats: &mut FilesStat,
        file_name: String,
        audio_info: &'a CommonAudioInfo,
    ) {
        self.stats.files.analyzed += 1;
        file_stats.analyzed += 1;

        // audio book genre set is a strong indicator
        // many discs and total discs is a strong indicator
        let mut has_enough_information = true;

        // artist + song name is key for bktree
        if audio_info.artist.is_empty() && audio_info.title.is_empty() {
            has_enough_information = false;
        }

        if has_enough_information {
            let key = AudioInfoKey::new(&audio_info.artist, &audio_info.title);

            // a) filter numbers / remove
            // b) use album name / substract it?

            let ref mut locked_container = data.lock().unwrap();
            let (vec_exact_match, vec_similarities) = locked_container.find(&key, TOLERANCE);
            if !vec_similarities.is_empty() {
                trace!("close: {:?} to {:?},", &vec_similarities, &key);
            }
            // if exact match, don't insert!!
            if vec_exact_match.is_empty() {
                // todo: also decide when to not add and insert then

                let key = key.clone();
                let audio_info = AudioInfo {
                    duration: audio_info.duration,
                    album: audio_info
                        .album
                        .as_ref()
                        .unwrap_or(&"no album".to_string())
                        .to_string(),
                    file_name: file_name.to_string(),
                };
                let value = Box::new(audio_info.clone());
                let mem_size = mem::size_of_val(&key) + mem::size_of_val(&value);
                locked_container.insert(key.clone(), value);
                // send to ipc
                ipc_sender
                    .send(IPC::PublishSingleAudioDataRecord(key, audio_info))
                    .unwrap_or_else(|e| warn!("something went very wrong {}!", e));
                self.stats.memory += mem_size as u64;
            } else {
                // exact match with certain AudioInfo
                self.stats.files.duplicates += vec_exact_match.len() as u32;
                for new_audio_info in vec_exact_match {
                    let time_distance = new_audio_info.duration - audio_info.duration;
                    if time_distance > Duration::from_secs(0) {
                        trace!(
                            "same: but time differs {:?} seconds with album name old '{}' and new: '{}'!",
                            time_distance,
                            new_audio_info.album,
                            audio_info
                                .album
                                .as_ref()
                                .unwrap_or(&"no album".to_string())
                                .to_string(),
                       );
                    }
                }
            }
        }
    }
}
