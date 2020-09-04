/// The net will be represented by a swarm as in libp2p
/// https://docs.rs/libp2p/latest/libp2p/swarm/index.html.
///
/// Because swarm is the manager a state machine as before could be replaced, also the working
/// Mdns Server/Client can just be transparently added used in the behavior of this network.
/// As transport layer, an experimental but prospering protocol called "noise protocol" will
/// be used.
/// The network communication is basically an actor system just as used here in the webui
/// with actix actor, which btw could maybe also replaced by the websocket protocol from
/// libp2p, but for now it will stay in a nice, small http server.
///
/// The noise protocol to be used
/// (http://noiseprotocol.org/)
///
use super::{
    super::{
        data::collection::AudioInfo,
        net::peer_representation::{self, PeerRepresentation},
    },
    sm_behaviour::{SMBehaviour, SMOutEvents},
    IPC,
};
use async_std::io;
use bincode;
use libp2p::{
    kad::{
        record,
        store::{MemoryStore, RecordStore},
        Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult, Quorum, Record,
    },
    mdns::{Mdns, MdnsEvent},
    pnet::{PnetConfig, PreSharedKey},
    swarm::NetworkBehaviourEventProcess,
    yamux::Config as YamuxConfig,
    NetworkBehaviour,
};
use libp2p_core::{
    either::EitherTransport, identity, transport::upgrade, PeerId, StreamMuxer, Transport,
};
use libp2p_noise::{Keypair, NoiseConfig, X25519Spec};
use libp2p_tcp::TcpConfig;
use std::{error::Error, time::Duration};

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
enum MkadKeys {
    KeyForPeerFinished(PeerRepresentation),
    SingleAudioRecord(AudioInfo),
}

/// The swarm injected behavior is the key element for the whole communication
/// See https://docs.rs/libp2p/0.21.1/libp2p/swarm/trait.NetworkBehaviour.html for more
#[derive(NetworkBehaviour)]
pub struct AdbfBehavior {
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: Mdns,
    pub sm_behaviour: SMBehaviour,
}
impl NetworkBehaviourEventProcess<MdnsEvent> for AdbfBehavior {
    // Called when `mdns` produces an event.
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                // most probably not the same PeerId ;-) but since it is multiaddress
                // its almost impossible to have 4 times the same random multiaddress
                let mut old_display_peer = PeerId::random();
                for (peer_id, multiaddr) in list {
                    self.sm_behaviour.mdns_new_peer(&peer_id, &multiaddr);
                    self.kademlia.add_address(&peer_id, multiaddr);
                    if old_display_peer != peer_id {
                        self.check_new_peer_actions(&peer_id);
                        old_display_peer = peer_id.clone();
                    }
                }
            }
            MdnsEvent::Expired(expired_addresses) => {
                for (peer_id, multi_addr) in expired_addresses {
                    self.kademlia.remove_address(&peer_id, &multi_addr);
                    self.sm_behaviour.mdns_remove(&peer_id);
                }
            }
        }
    }
}
impl NetworkBehaviourEventProcess<KademliaEvent> for AdbfBehavior {
    // Called when `kademlia` produces an event.
    fn inject_event(&mut self, message: KademliaEvent) {
        match message {
            KademliaEvent::QueryResult { result, .. } => match result {
                QueryResult::GetRecord(get_record) => match get_record {
                    Ok(ok) => {
                        for PeerRecord {
                            record: Record { key, value, .. },
                            ..
                        } in ok.records
                        {
                            self.retrieve_record(key, value);
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
            _ => (), // trace!("kademlie routing events occured!"),
        }
    }
}

impl NetworkBehaviourEventProcess<SMOutEvents> for AdbfBehavior {
    // Called when SM produces an event.
    fn inject_event(&mut self, event: SMOutEvents) {
        // send whole event
        match event {
            SMOutEvents::ForwardSM(_sm_event) => {
                // there is none yet
                // todo: add it!
            }
            SMOutEvents::ForwardIPC(ipc_event) => {
                // the key is to avoid duplicate, so the key
                // is a hash of the message itself
                let own_peer = peer_representation::peer_to_hash(&self.sm_behaviour.own_peer());

                // must be used in each ipc-event
                let bin_key;

                let serialize_message = match ipc_event {
                    IPC::DoneSearching(nr) => {
                        bin_key = Self::key_writer(MkadKeys::KeyForPeerFinished(own_peer));

                        // try to read old value
                        let mut value_to_send = nr;
                        if let Some(already_peer_finished_record) =
                            self.kademlia.store_mut().get(&bin_key)
                        {
                            let found: u32 =
                                bincode::deserialize(already_peer_finished_record.value.as_ref())
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
                    IPC::PublishSingleAudioDataRecord(audio_info) => {
                        // a single audio data
                        bin_key = Self::key_writer(MkadKeys::SingleAudioRecord(audio_info.clone()));
                        if let Some(already_audio_record) = self.kademlia.store_mut().get(&bin_key)
                        {
                            let already_audio_data: Result<AudioInfo, bincode::Error> =
                                bincode::deserialize(already_audio_record.value.as_ref());
                            if let Ok(found_and_deserializable) = already_audio_data {
                                info!(
                                    "This record was already found somewhere else, and put as {}!",
                                    found_and_deserializable.file_name
                                );
                            } else {
                                info!(
                                    "This record was already there and not even de-serializable!"
                                );
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
                    self.kademlia
                        .put_record(record, Quorum::One)
                        .expect("Failed to store record in kademlia locally.");
                } else {
                    warn!("not possible to send IPC through kademlia!");
                }
            }
        }
    }
}

pub fn build_noise_transport(
    key_pair: &identity::Keypair,
    psk: Option<PreSharedKey>,
) -> impl Transport<
    Output = (
        PeerId,
        impl StreamMuxer<
                OutboundSubstream = impl Send,
                Substream = impl Send,
                Error = impl Into<io::Error>,
            > + Send
            + Sync,
    ),
    Error = impl Error + Send,
    Listener = impl Send,
    Dial = impl Send,
    ListenerUpgrade = impl Send,
> + Clone {
    let dh_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&key_pair)
        .unwrap();
    let noise_config = NoiseConfig::xx(dh_keys).into_authenticated();
    let yamux_config = YamuxConfig::default();

    let base_transport = TcpConfig::new().nodelay(true);
    let maybe_encrypted = match psk {
        Some(psk) => EitherTransport::Left(
            base_transport.and_then(move |socket, _| PnetConfig::new(psk).handshake(socket)),
        ),
        None => EitherTransport::Right(base_transport),
    };
    maybe_encrypted
        .upgrade(upgrade::Version::V1)
        .authenticate(noise_config)
        .multiplex(yamux_config)
        .timeout(Duration::from_secs(20))
}

impl AdbfBehavior {
    fn retrieve_record(&mut self, key: record::Key, _value: Vec<u8>) {
        warn!("..............................................");
        if let Ok(fits_mkad_keys) = Self::key_reader(&key) {
            match fits_mkad_keys {
                MkadKeys::KeyForPeerFinished(peer_hash) => {
                    // todo: continue here
                    info!("key for peer finished of '{}' retrieved!", peer_hash);
                }
                MkadKeys::SingleAudioRecord(audio_info) => {
                    info!(
                        "new audio data with name '{}' retrieved!",
                        audio_info.file_name
                    );
                }
            }
        } else {
            error!("unknown MkadKeys format");
        }
    }

    fn key_writer(internal_key: MkadKeys) -> record::Key {
        let bin_key = bincode::serialize(&internal_key).unwrap();
        record::Key::new(&bin_key)
    }
    fn key_reader(
        key_record: &record::Key,
    ) -> Result<MkadKeys, std::boxed::Box<bincode::ErrorKind>> {
        bincode::deserialize(key_record.as_ref())
    }

    fn get_key_finished(&mut self, key: MkadKeys) -> Result<u32, ()> {
        let serialized_key = Self::key_writer(key);
        match self.kademlia.store_mut().get(&serialized_key) {
            Some(good_query) => {
                let found: u32 =
                    bincode::deserialize(good_query.value.as_ref()).unwrap_or_else(|_| {
                        error!("value in store is not what expected!");
                        0
                    });
                Ok(found)
            }
            None => Err(()),
        }
    }

    /// Looks into kademlia data and returns if already finished number
    /// has been submitted.
    fn check_if_peer_finished(&mut self, peer_id: &PeerId) -> Result<u32, ()> {
        let peer_hash = peer_representation::peer_to_hash(peer_id);
        let query_key = MkadKeys::KeyForPeerFinished(peer_hash);
        self.get_key_finished(query_key)
    }

    fn check_new_peer_actions(&mut self, peer_id: &PeerId) {
        if *peer_id == self.sm_behaviour.own_peer() {
            warn!("own instance finished ... not interesting, should not happen!");
        } else {
            if let Ok(count) = self.check_if_peer_finished(&peer_id) {
                self.sm_behaviour.update_peer_data(&peer_id, count);
            }
        }
    }
}
