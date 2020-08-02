/// a very small mod just for ui data send by net. It is important to
/// not send all discovery blindly (e.g. duplicates)
use libp2p_core::{Multiaddr, PeerId};
use std::{collections::HashSet, sync::mpsc::Sender};

use super::super::{
    ctrl::{self, ForwardNetMessage, UiPeer, UiUpdateMsg},
    net::peer_representation,
    net::sm::*,
};

pub struct UiData {
    sender: Option<Sender<UiUpdateMsg>>,
    ui_shown_peers: HashSet<PeerId>,
}
impl UiData {
    pub fn new(sender: Option<Sender<UiUpdateMsg>>) -> Self {
        Self {
            sender,
            ui_shown_peers: HashSet::new(),
        }
    }

    pub fn has_peer(&mut self, peer_id: &PeerId) -> bool {
        let ref collection = self.ui_shown_peers;
        collection.get(peer_id).is_some()
    }

    pub fn register_address(&mut self, peer_id: &PeerId, multi_addresses: &Multiaddr) {
        let ref mut collection = self.ui_shown_peers;
        if collection.get(peer_id).is_none() {
            // add
            collection.insert(peer_id.clone());
            trace!(
                "found new peer {}",
                peer_representation::peer_to_hash_string(peer_id)
            );
            // and send
            if let Some(ctrl_sender) = &self.sender {
                let addr_as_string = multi_addresses.iter().map(|x| x.to_string()).collect();

                ctrl_sender
                    .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::Add(
                        UiPeer {
                            id: peer_id.clone(),
                            addresses: addr_as_string,
                        },
                    )))
                    .unwrap_or_else(|e| error!("use one: {}", e));
            }
        } else {
            error!("Hey, the StateMachine should have made this impossible!!!");
        }
    }
    pub fn unregister_address(&mut self, peer_id: &PeerId) {
        let ref mut collection = self.ui_shown_peers;
        if collection.remove(peer_id) {
            trace!(
                "removed peer {}",
                peer_representation::peer_to_hash_string(peer_id)
            );
            if let Some(ctrl_sender) = &self.sender {
                ctrl_sender
                    .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::Delete(
                        peer_id.clone(),
                    )))
                    .unwrap_or_else(|e| error!("use one: {}", e));
            }
        } else {
            warn!("Trying to remove something which wasn't there ...");
        }
    }

    pub fn update_peer_data(&mut self, update_data: &UpdateData) {
        if self.has_peer(&update_data.id) {
            if let Some(ctrl_sender) = &self.sender {
                ctrl_sender
                    .send(ctrl::UiUpdateMsg::PeerSearchFinished(
                        update_data.id.clone(),
                        update_data.count,
                    ))
                    .unwrap_or_else(|e| error!("use one: {}", e));
            }
        } else {
            warn!(
                "Peer {} is not known!",
                peer_representation::peer_to_hash_string(&update_data.id)
            );
        }
    }
}
