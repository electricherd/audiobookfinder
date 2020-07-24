use libp2p_core::PeerId;
use std::{collections::hash_map::DefaultHasher, hash::Hasher};

/// Since peer lacks some functionality, PeerRepresentation is for convenience
/// (shortening, Serialization)
pub type PeerRepresentation = u64;
pub fn peer_to_hash(peer_id: &PeerId) -> PeerRepresentation {
    let mut hasher = DefaultHasher::default();
    hasher.write(peer_id.as_ref());
    hasher.finish()
}
pub fn peer_to_hash_string(peer_id: &PeerId) -> String {
    std::format!("{:x?}", peer_to_hash(peer_id))
}
