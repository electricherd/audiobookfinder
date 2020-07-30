use id3::Tag as id3tag;
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
    fn read_tag_from(file: &mut BufReader<File>) -> Result<CommonAudioInfo, String>;
}

pub struct MP4TagReader;
impl MP4TagReader {
    pub fn read_tag_from(file_buffer: &mut BufReader<File>) -> Result<CommonAudioInfo, String> {
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
}

pub struct ID3TagReader;
impl ID3TagReader {
    pub fn read_tag_from(file_buffer: &mut BufReader<File>) -> Result<CommonAudioInfo, String> {
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
}
