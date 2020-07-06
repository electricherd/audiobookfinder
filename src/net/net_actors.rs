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
use super::ui_data::UiData;

use async_std::io;
use libp2p::{
    kad::{
        record::store::MemoryStore, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult,
        Record,
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

#[derive(NetworkBehaviour)]
pub struct CustomBehaviour {
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub ui_data: UiData,
}
impl NetworkBehaviourEventProcess<MdnsEvent> for CustomBehaviour {
    // Called when `mdns` produces an event.
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer_id, multiaddr) in list {
                    trace!("found new peer {}", peer_id.to_string());
                    self.kademlia.add_address(&peer_id, multiaddr);
                    self.ui_data.register_address(&peer_id);
                }
            }
            MdnsEvent::Expired(expired_addresses) => {
                for (peer_id, multi_addr) in expired_addresses {
                    self.kademlia.remove_address(&peer_id, &multi_addr);
                    self.ui_data.unregister_address(&peer_id);
                }
            }
        }
    }
}
impl NetworkBehaviourEventProcess<KademliaEvent> for CustomBehaviour {
    // Called when `kademlia` produces an event.
    fn inject_event(&mut self, message: KademliaEvent) {
        match message {
            KademliaEvent::QueryResult { result, .. } => match result {
                QueryResult::GetRecord(Ok(ok)) => {
                    for PeerRecord {
                        record: Record { key, value, .. },
                        ..
                    } in ok.records
                    {
                        info!(
                            "Got record {:?} {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap(),
                            std::str::from_utf8(&value).unwrap(),
                        );
                    }
                }
                QueryResult::GetRecord(Err(err)) => {
                    error!("Failed to get record: {:?}", err);
                }
                QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                    trace!(
                        "Successfully put record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                QueryResult::PutRecord(Err(err)) => {
                    error!("Failed to put record: {:?}", err);
                }
                _ => {}
            },
            _ => {}
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
