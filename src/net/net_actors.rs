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
use super::sm_behaviour::{SMBehaviour, SMOutEvents};
use super::IPC;

use async_std::io;
use bincode;
use libp2p::{
    kad::{
        record, store::MemoryStore, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult,
        Quorum, Record,
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
    KeyForAllPeerFinished,
    KeyForIPC,
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
                for (peer_id, multiaddr) in list {
                    self.sm_behaviour.mdns_new_peer(&peer_id, &multiaddr);
                    self.kademlia.add_address(&peer_id, multiaddr);
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
                        trace!(
                            "Successfully put record {:?}",
                            Self::key_reader(&key).unwrap()
                        );
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
            SMOutEvents::ForwardSM(sm_event) => {
                // there is none yet
            }
            SMOutEvents::ForwardIPC(ipc_event) => {
                // the key is to avoid duplicate, so the key
                // is a hash of the message itself
                match ipc_event {
                    IPC::DoneSearching(_) => {
                        info!("forward ipc {:?}", ipc_event);
                        let bin_message = bincode::serialize(&ipc_event).unwrap();
                        let bin_key = Self::key_writer(MkadKeys::KeyForIPC);

                        let record = Record {
                            key: bin_key,
                            value: bin_message,
                            publisher: None,
                            expires: None,
                        };
                        // write out
                        self.kademlia
                            .put_record(record, Quorum::One)
                            .expect("Failed to store record locally.");
                    }
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
    fn retrieve_record(&mut self, key: record::Key, value: Vec<u8>) {
        warn!("..............................................");
        if let Ok(fits_mkad_keys) = Self::key_reader(&key) {
            match fits_mkad_keys {
                MkadKeys::KeyForAllPeerFinished => (),
                MkadKeys::KeyForIPC => info!("ipc message came up!"),
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

    fn add_myself_to_peers_done(&mut self) {
        // see if I am already in there
        let query_key = Self::key_writer(MkadKeys::KeyForAllPeerFinished);
        let kad_record = self.kademlia.get_record(&query_key, Quorum::Majority);
    }
}
