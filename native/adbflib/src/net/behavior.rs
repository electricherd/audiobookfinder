//! The net will be represented by a swarm as in libp2p
//! https://docs.rs/libp2p/latest/libp2p/swarm/index.html.
//!
//! Because swarm is the manager a state machine as before could be replaced, also the working
//! Mdns Server/Client can just be transparently added used in the behavior of this network.
//! As transport layer, an experimental but prospering protocol called "noise protocol" will
//! be used.
//! The network communication is basically an actor system just as used here in the webui
//! with actix actor, which btw could maybe also replaced by the websocket protocol from
//! libp2p, but for now it will stay in a nice, small http server.
//!
//! The noise protocol being used
//! (http://noiseprotocol.org/)
use super::{
    sm_behaviour::{SMBehaviour, SMOutEvents},
    storage::NetStorage,
    subs::peer_representation,
};
use libp2p::{
    core::{
        either::EitherTransport, identity::Keypair, muxing::StreamMuxerBox, transport,
        transport::upgrade, PeerId, Transport,
    },
    kad::{store::MemoryStore, Kademlia, KademliaEvent},
    mdns::{Mdns, MdnsEvent},
    noise::{self, NoiseConfig, X25519Spec},
    pnet::{PnetConfig, PreSharedKey},
    swarm::NetworkBehaviourEventProcess,
    tcp::TcpConfig,
    yamux::YamuxConfig,
    NetworkBehaviour,
};
use std::time::Duration;

/// The swarm injected behavior is the key element for the whole communication
/// See https://docs.rs/libp2p/latest/libp2p/swarm/trait.NetworkBehaviour.html for more
#[derive(NetworkBehaviour)]
pub struct AdbfBehavior {
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: Mdns,
    pub sm_behaviour: SMBehaviour,
    #[behaviour(ignore)]
    pub storage: NetStorage,
}

/// MDns Part of AdbfBehavior
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
                        self.storage.inc();
                    }
                }
            }
            MdnsEvent::Expired(expired_addresses) => {
                for (peer_id, multi_addr) in expired_addresses {
                    self.kademlia.remove_address(&peer_id, &multi_addr);
                    self.sm_behaviour.mdns_remove(&peer_id);
                    self.storage.dec();
                }
            }
        }
    }
}

/// Kademlia Part of AdbfBehavior
impl NetworkBehaviourEventProcess<KademliaEvent> for AdbfBehavior {
    // Called when `kademlia` produces an event.
    fn inject_event(&mut self, message: KademliaEvent) {
        self.storage.on_retrieve(message);
    }
}

/// Own injected state machine (SMOutEvents) part of AdbfBehavior
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

                // write ipc message to net storage
                self.storage
                    .write_ipc(&mut self.kademlia, own_peer, ipc_event);
            }
        }
    }
}

/// Build up the transport layer
pub fn build_noise_transport(
    key_pair: &Keypair,
    psk: Option<PreSharedKey>,
) -> transport::Boxed<(PeerId, StreamMuxerBox)> {
    let noise_keys = noise::Keypair::<X25519Spec>::new()
        .into_authentic(key_pair)
        .unwrap();
    let noise_config = NoiseConfig::xx(noise_keys).into_authenticated();
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
        .boxed()
}

impl AdbfBehavior {
    fn check_new_peer_actions(&mut self, peer_id: &PeerId) {
        if *peer_id == self.sm_behaviour.own_peer() {
            warn!("own instance finished ... not interesting, should not happen!");
        } else {
            if let Ok(count) = NetStorage::check_if_peer_finished(&mut self.kademlia, &peer_id) {
                self.sm_behaviour.update_peer_data(&peer_id, count);
            }
        }
    }
}
