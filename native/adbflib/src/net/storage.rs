//! wraps up "net storage" which is implementing the kademlia functionality.
use super::{
    super::data::{
        audio_info::{AudioInfo, AudioInfoKey},
        ipc::{IFCollectionOutputData, IPC},
    },
    subs::peer_representation::{self, PeerRepresentation},
};
use bincode;
use libp2p::{
    core::PeerId,
    kad::{
        record,
        store::{MemoryStore, RecordStore},
        Kademlia,
        KademliaEvent::{self, OutboundQueryCompleted},
        PeerRecord, PutRecordOk, QueryResult, Quorum, Record,
    },
};

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MkadKeys {
    KeyForPeerFinished(PeerRepresentation),
    SingleAudioRecord(AudioInfoKey),
}

/// Capsulates NetStorage activity internally
/// using Kademlia.
pub struct NetStorage {
    nr_peers: usize,
}

impl NetStorage {
    pub fn new() -> Self {
        Self { nr_peers: 0 }
    }
    pub fn inc(&mut self) {
        self.nr_peers += 1;
    }
    pub fn dec(&mut self) {
        if self.nr_peers > 0 {
            self.nr_peers -= 1;
        } else {
            error!("Trying to remove more peers as are known. Will not happen")
        }
    }
    pub fn peers(&self) -> usize {
        self.nr_peers
    }

    pub fn write_ipc(
        &mut self,
        kademlia: &mut Kademlia<MemoryStore>,
        own_peer: u64,
        ipc_event: IPC,
    ) {
        if self.peers() == 0 {
            // todo: there is no peer to really write to?! Need to check how kademlia works!
            warn!("There are no known peers that would react to writing to net storage yet!");
        } else {
            let bin_key;
            let serialize_message = match ipc_event {
                IPC::DoneSearching(out_data) => {
                    bin_key = Self::key_writer(MkadKeys::KeyForPeerFinished(own_peer));
                    // try to read old value
                    let value_to_send = out_data;
                    if let Some(already_peer_finished_record) = kademlia.store_mut().get(&bin_key) {
                        let try_collection_output: Result<IFCollectionOutputData, bincode::Error> =
                            bincode::deserialize(already_peer_finished_record.value.as_ref());
                        if let Ok(found_and_deserializable) = try_collection_output {
                            info!(
                                "This record was already found somewhere else, and has {:?}!",
                                found_and_deserializable
                            );
                        }
                    } else {
                        trace!(
                            "this key {:?} was not yet set in the kademlia store with value!",
                            bin_key
                        );
                    }
                    Some(bincode::serialize(&value_to_send).unwrap())
                }
                IPC::PublishSingleAudioDataRecord(audio_key, audio_info) => {
                    // a single audio data
                    bin_key = Self::key_writer(MkadKeys::SingleAudioRecord(audio_key));
                    if let Some(already_audio_record) = kademlia.store_mut().get(&bin_key) {
                        let already_audio_data: Result<AudioInfo, bincode::Error> =
                            bincode::deserialize(already_audio_record.value.as_ref());
                        if let Ok(found_and_deserializable) = already_audio_data {
                            info!(
                                "This record was already found somewhere else, and put as {}!",
                                found_and_deserializable.file_name
                            );
                        } else {
                            info!("This record was already there and not even de-serializable!");
                        }
                        None
                    } else {
                        // that is new and should be put
                        Some(bincode::serialize(&audio_info).unwrap())
                    }
                }
            };

            // check if it is ok to send
            if let Some(bin_message) = serialize_message {
                let record = Record {
                    key: bin_key,
                    value: bin_message,
                    publisher: None,
                    expires: None,
                };

                // write out
                kademlia
                    .put_record(record, Quorum::One)
                    .expect("Failed to store record in kademlia locally.");
            } else {
                warn!("not possible to send IPC through kademlia!");
            }
        }
    }

    /// Looks into kademlia data and returns if already finished number
    /// has been submitted.
    pub fn check_if_peer_finished(
        kademlia: &mut Kademlia<MemoryStore>,
        peer_id: &PeerId,
    ) -> Result<IFCollectionOutputData, ()> {
        let peer_hash = peer_representation::peer_to_hash(peer_id);
        let query_key = MkadKeys::KeyForPeerFinished(peer_hash);
        Self::get_data_finished(kademlia, query_key)
    }

    pub fn on_retrieve(&self, event: KademliaEvent) {
        match event {
            OutboundQueryCompleted {
                id: _id, result, ..
            } => match result {
                QueryResult::GetRecord(get_record) => match get_record {
                    Ok(ok) => {
                        for PeerRecord {
                            record: Record { key, value, .. },
                            ..
                        } in ok.records
                        {
                            Self::retrieve_record(key, value);
                        }
                    }
                    Err(err) => {
                        error!("Failed to get record: {:?}", err);
                    }
                },
                QueryResult::PutRecord(put_record) => match put_record {
                    Ok(PutRecordOk { key }) => {
                        let raw_key = Self::key_reader(&key);
                        match raw_key {
                            Ok(deserialized) => match deserialized {
                                MkadKeys::KeyForPeerFinished(peer_hash) => info!(
                                    "Successfully put record key KeyForPeerFinished for peer {:?}!",
                                    peer_representation::peer_hash_to_string(&peer_hash)
                                ),
                                MkadKeys::SingleAudioRecord(_audio_info) => {
                                    info!("Successfully put record key SingleAudioRecord!")
                                }
                            },
                            Err(_) => {
                                error!("key could not be verified!");
                            }
                        }
                    }
                    Err(err) => {
                        error!("Failed to put record: {:?}", err);
                    }
                },
                _ => trace!("other kademlie query results arrived?!"),
            },
            _ => {
                error!("outbound query expected");
            }
        }
    }

    fn key_writer(internal_key: MkadKeys) -> record::Key {
        let bin_key = bincode::serialize(&internal_key).unwrap();
        record::Key::new(&bin_key)
    }

    fn retrieve_record(key: record::Key, _value: Vec<u8>) {
        warn!("..............................................");
        if let Ok(fits_mkad_keys) = Self::key_reader(&key) {
            match fits_mkad_keys {
                MkadKeys::KeyForPeerFinished(peer_hash) => {
                    // todo: continue here
                    info!("key for peer finished of '{}' retrieved!", peer_hash);
                }
                MkadKeys::SingleAudioRecord(audio_key) => {
                    info!("new audio data with key '{}' retrieved!", &audio_key.get());
                }
            }
        } else {
            error!("unknown MkadKeys format");
        }
    }

    fn key_reader(
        key_record: &record::Key,
    ) -> Result<MkadKeys, std::boxed::Box<bincode::ErrorKind>> {
        bincode::deserialize(key_record.as_ref())
    }

    fn get_data_finished(
        kademlia: &mut Kademlia<MemoryStore>,
        key: MkadKeys,
    ) -> Result<IFCollectionOutputData, ()> {
        let serialized_key = Self::key_writer(key);
        match kademlia.store_mut().get(&serialized_key) {
            Some(good_query) => {
                let value: Result<IFCollectionOutputData, bincode::Error> =
                    bincode::deserialize(good_query.value.as_ref());
                value.map_err(|_e| ())
            }
            None => Err(()),
        }
    }
}
