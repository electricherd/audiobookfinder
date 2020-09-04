//! wraps up "net storage" which is implementing the kademlia functionality.
use super::{
    super::data::{
        audio_info::{AudioInfo, AudioInfoKey},
        ipc::IPC,
    },
    subs::peer_representation::{self, PeerRepresentation},
};
use bincode;
use libp2p::kad::{
    record,
    store::{MemoryStore, RecordStore},
    Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult, Quorum, Record,
};
use libp2p_core::PeerId;

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MkadKeys {
    KeyForPeerFinished(PeerRepresentation),
    SingleAudioRecord(AudioInfoKey),
}

pub fn write_ipc(kademlia: &mut Kademlia<MemoryStore>, own_peer: u64, ipc_event: IPC) {
    let bin_key;
    let serialize_message = match ipc_event {
        IPC::DoneSearching(nr) => {
            bin_key = key_writer(MkadKeys::KeyForPeerFinished(own_peer));

            // try to read old value
            let mut value_to_send = nr;
            if let Some(already_peer_finished_record) = kademlia.store_mut().get(&bin_key) {
                let found: u32 = bincode::deserialize(already_peer_finished_record.value.as_ref())
                    .unwrap_or_else(|_| {
                        error!("value in store is not what expected!");
                        0
                    });
                value_to_send += found;
            } else {
                trace!(
                    "this key was not yet set in the kademlia store with value {}!",
                    nr
                );
            }
            Some(bincode::serialize(&value_to_send).unwrap())
        }
        IPC::PublishSingleAudioDataRecord(audio_key, audio_info) => {
            // a single audio data
            bin_key = key_writer(MkadKeys::SingleAudioRecord(audio_key));
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

/// Looks into kademlia data and returns if already finished number
/// has been submitted.
pub fn check_if_peer_finished(
    kademlia: &mut Kademlia<MemoryStore>,
    peer_id: &PeerId,
) -> Result<u32, ()> {
    let peer_hash = peer_representation::peer_to_hash(peer_id);
    let query_key = MkadKeys::KeyForPeerFinished(peer_hash);
    get_key_finished(kademlia, query_key)
}

pub fn on_retrieve(message: KademliaEvent) {
    match message {
        KademliaEvent::QueryResult { result, .. } => match result {
            QueryResult::GetRecord(get_record) => match get_record {
                Ok(ok) => {
                    for PeerRecord {
                        record: Record { key, value, .. },
                        ..
                    } in ok.records
                    {
                        retrieve_record(key, value);
                    }
                }
                Err(err) => {
                    error!("Failed to get record: {:?}", err);
                }
            },
            QueryResult::PutRecord(put_record) => match put_record {
                Ok(PutRecordOk { key }) => {
                    let raw_key = key_reader(&key);
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
        _ => (), // trace!("kademlie routing events occured!"),
    }
}

fn key_writer(internal_key: MkadKeys) -> record::Key {
    let bin_key = bincode::serialize(&internal_key).unwrap();
    record::Key::new(&bin_key)
}

fn retrieve_record(key: record::Key, _value: Vec<u8>) {
    warn!("..............................................");
    if let Ok(fits_mkad_keys) = key_reader(&key) {
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

fn key_reader(key_record: &record::Key) -> Result<MkadKeys, std::boxed::Box<bincode::ErrorKind>> {
    bincode::deserialize(key_record.as_ref())
}

fn get_key_finished(kademlia: &mut Kademlia<MemoryStore>, key: MkadKeys) -> Result<u32, ()> {
    let serialized_key = key_writer(key);
    match kademlia.store_mut().get(&serialized_key) {
        Some(good_query) => {
            let found: u32 = bincode::deserialize(good_query.value.as_ref()).unwrap_or_else(|_| {
                error!("value in store is not what expected!");
                0
            });
            Ok(found)
        }
        None => Err(()),
    }
}
