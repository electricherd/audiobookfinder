/// Module with interface to different tag readers
use id3::Tag as id3tag;
use metaflac::{block::Block, Tag as flactag};
use mp3_metadata;
use mp4ameta::Tag as mp4tag;
use std::{fs::File, io::BufReader, time::Duration};

#[allow(dead_code)]
#[derive(Clone)]
pub struct CommonAudioInfo {
    pub title: String,
    pub artist: String,
    pub duration: Duration,
    pub album: Option<String>,
    pub track: Option<u32>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub disc: Option<u32>,
    pub total_discs: Option<u32>,
    pub total_tracks: Option<u32>,
    pub year: Option<i32>,
}

/// Trait to ensure same calls
pub trait TagReader<'a> {
    fn read_tag_from(&self, file: &mut BufReader<File>) -> Result<CommonAudioInfo, String>;
    fn known_suffixes(&self) -> Vec<&'a str>;
}

pub struct MP4TagReader;
impl<'a> TagReader<'a> for MP4TagReader {
    fn read_tag_from(&self, file_buffer: &mut BufReader<File>) -> Result<CommonAudioInfo, String> {
        match mp4tag::read_from(file_buffer.get_mut()) {
            Ok(tag) => {
                // track info need extra treatment in mp4ameta
                let try_track_info = tag.track_number();
                let (track, total_tracks) = match try_track_info {
                    Some(good_track_info) => {
                        let (track, total_tracks) = good_track_info;
                        (Some(track as u32), Some(total_tracks as u32))
                    }
                    None => (None, None),
                };
                // year needs extra treatment
                let year = tag
                    .year()
                    .map_or(None, |good_string| good_string.parse::<i32>().ok());

                let info = CommonAudioInfo {
                    title: tag.title().unwrap_or("").to_string(),
                    artist: tag.artist().unwrap_or("").to_string(),
                    duration: Duration::from_secs(tag.duration().unwrap_or(0.0) as u64),
                    album: tag.album().map(|op| op.to_string()),
                    track,
                    album_artist: tag.album_artist().map(|op| op.to_string()),
                    genre: tag.genre().map(|op| op.to_string()),
                    disc: None,        // no supported
                    total_discs: None, // no supported
                    total_tracks,
                    year,
                };
                Ok(info)
            }
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    fn known_suffixes(&self) -> Vec<&'a str> {
        vec!["mp4"]
    }
}

pub struct ID3TagReader;
impl<'a> TagReader<'a> for ID3TagReader {
    fn read_tag_from(&self, file_buffer: &mut BufReader<File>) -> Result<CommonAudioInfo, String> {
        match id3tag::read_from(file_buffer.get_mut()) {
            Ok(tag) => {
                // write into common audio info that can be analyzed
                let info = CommonAudioInfo {
                    title: tag.title().unwrap_or("").to_string(),
                    artist: tag.artist().unwrap_or("").to_string(),
                    duration: Duration::from_secs(tag.duration().unwrap_or(0) as u64),
                    album: tag.album().map(|s| s.to_string()),
                    track: tag.track(),
                    album_artist: tag.album_artist().map(|st| st.to_string()),
                    genre: tag.genre().map(|s| s.to_string()),
                    disc: tag.disc(),
                    total_discs: tag.total_discs(),
                    total_tracks: tag.total_tracks(),
                    year: tag.year(),
                };
                Ok(info)
            }
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    fn known_suffixes(&self) -> Vec<&'a str> {
        vec!["mpeg"]
    }
}

pub struct FlacTagReader;
impl<'a> TagReader<'a> for FlacTagReader {
    fn read_tag_from(&self, file_buffer: &mut BufReader<File>) -> Result<CommonAudioInfo, String> {
        match flactag::read_from(file_buffer) {
            Ok(tag_block) => {
                // try the first block
                let mut res_info = Err("vorbis comment block not found".to_string());
                for good_tag_block in tag_block.blocks() {
                    match good_tag_block {
                        Block::VorbisComment(tag) => {
                            // helper
                            let take_first_or_empty = |o: Option<&Vec<String>>| -> String {
                                (*o.map(|v_s| v_s.first().unwrap_or(&"".to_string()).clone())
                                    .unwrap_or("".to_string()))
                                .to_string()
                            };
                            let take_first_or_option = |o: Option<&Vec<String>>| -> Option<String> {
                                o.map(|v_s| Some(v_s.first().unwrap_or(&"".to_string()).clone()))
                                    .unwrap_or(None)
                            };

                            res_info = Ok(CommonAudioInfo {
                                title: take_first_or_empty(tag.title()),
                                artist: take_first_or_empty(tag.artist()),
                                // todo: duration somewhere else
                                duration: Duration::from_secs(0),
                                album: take_first_or_option(tag.album()),
                                track: tag.track(),
                                album_artist: take_first_or_option(tag.album_artist()),
                                genre: take_first_or_option(tag.genre()),
                                disc: Some(0), // tag.disc(),
                                // todo: discs, total discs, year somewhere else??
                                total_discs: Some(0), // tag.total_discs(),
                                total_tracks: tag.total_tracks(),
                                year: Some(0), //tag.year(),
                            });
                        }
                        _ => (),
                    }
                }
                res_info
            }
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    fn known_suffixes(&self) -> Vec<&'a str> {
        vec!["flac", "x-vorbis+ogg"]
    }
}

pub struct MP3TagReader;
impl<'a> TagReader<'a> for MP3TagReader {
    fn read_tag_from(&self, file_buffer: &mut BufReader<File>) -> Result<CommonAudioInfo, String> {
        match mp3_metadata::read_from_slice(file_buffer.buffer()) {
            Ok(metadata) => {
                match metadata.tag {
                    Some(tag) => {
                        // write into common audio info that can be analyzed
                        let info = CommonAudioInfo {
                            title: tag.title,
                            artist: tag.artist,
                            duration: Duration::from_secs(0),
                            album: Some(tag.album),
                            track: None,
                            album_artist: None,
                            genre: Some(format!("{:?}", tag.genre)),
                            disc: None,
                            total_discs: None,
                            total_tracks: None,
                            year: Some(tag.year as i32),
                        };
                        Ok(info)
                    }
                    None => Err("no audio tag found".to_string()),
                }
            }
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    fn known_suffixes(&self) -> Vec<&'a str> {
        vec!["mpeg", "mp3"]
    }
}
