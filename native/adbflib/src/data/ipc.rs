//! IPC module will hold all massive (that is why IPC) internal messages
//! which occur due to data collection, its start and its end.
use super::{
    audio_info::{AudioInfo, AudioInfoKey},
    IFInternalCollectionOutputData,
};

// todo: move to mod
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IFCollectionOutputData {
    pub nr_searched_files: u32,
    pub nr_found_songs: u32,
    pub nr_internal_duplicates: u32,
    pub size_of_data_in_kb: usize,
}

impl IFCollectionOutputData {
    pub fn from(internal: &IFInternalCollectionOutputData) -> Self {
        Self {
            nr_searched_files: internal.nr_searched_files,
            nr_found_songs: internal.nr_found_songs,
            nr_internal_duplicates: internal.nr_internal_duplicates,
            size_of_data_in_kb: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IPC {
    DoneSearching(IFCollectionOutputData),
    PublishSingleAudioDataRecord(AudioInfoKey, AudioInfo),
}
