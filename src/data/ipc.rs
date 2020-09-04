//! IPC module will hold all massive (that is why IPC) internal messages
//! which occur due to data collection, its start and its end.
use super::audio_info::{AudioInfo, AudioInfoKey};

#[derive(Serialize, Deserialize, Debug)]
pub enum IPC {
    DoneSearching(u32),
    PublishSingleAudioDataRecord(AudioInfoKey, AudioInfo),
}
