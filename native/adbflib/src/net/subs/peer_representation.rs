//! Holds the peer representation, which is a hash of the PeerId
//! which is also a hash or the public key inside.
//! It's for convenience and smaller footprint.
use libp2p::core::PeerId;
use std::{collections::hash_map::DefaultHasher, hash::Hasher};

/// Since peer lacks some functionality, PeerRepresentation is for convenience
/// (shortening, Serialization)
pub type PeerRepresentation = u64;

/// Return PeerRepresentation directly from PeerId
pub fn peer_to_hash(peer_id: &PeerId) -> PeerRepresentation {
    let mut hasher = DefaultHasher::default();
    hasher.write(&peer_id.to_bytes());
    hasher.finish()
}
/// Return a hex string hash representation of PeerId
pub fn peer_to_hash_string(peer_id: &PeerId) -> String {
    std::format!("{:x?}", peer_to_hash(peer_id))
}
/// Return a hex string hash reprentation of pure peer representation
pub fn peer_hash_to_string(peer: &PeerRepresentation) -> String {
    std::format!("{:x?}", peer)
}
