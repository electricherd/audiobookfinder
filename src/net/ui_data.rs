/// a very small mod just for ui data send by net. It is important to
/// not send all discovery blindly (e.g. duplicates)
use libp2p_core::{Multiaddr, PeerId};
use std::{collections::HashMap, sync::mpsc::Sender};

use super::super::ctrl::{self, ForwardNetMessage, UiPeer, UiUpdateMsg};

pub struct UiData {
    sender: Option<Sender<UiUpdateMsg>>,
    // todo: use, fill () with some nice data for webui functionality, etc.
    ui_shown_peers: HashMap<PeerId, ()>,
}
impl UiData {
    pub fn new(sender: Option<Sender<UiUpdateMsg>>) -> Self {
        Self {
            sender,
            ui_shown_peers: HashMap::new(),
        }
    }
    pub fn register_address(&mut self, peer_id: &PeerId, multi_addresses: &Multiaddr) {
        let ref mut collection = self.ui_shown_peers;
        if collection.get(peer_id).is_none() {
            // add
            collection.insert(peer_id.clone(), ());
            trace!("found new peer {}", peer_id.to_string());
            // and send
            if let Some(ctrl_sender) = &self.sender {
                let addr_as_string = multi_addresses.iter().map(|x| x.to_string()).collect();

                ctrl_sender
                    .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::Add(
                        UiPeer {
                            id: peer_id.to_string(),
                            addresses: addr_as_string,
                        },
                    )))
                    .unwrap_or_else(|e| error!("use one: {}", e));
            }
        }
    }
    pub fn unregister_address(&mut self, peer_id: &PeerId) {
        //
        let ref mut collection = self.ui_shown_peers;
        if collection.remove(peer_id).is_some() {
            trace!("removed peer {}", peer_id.to_string());
            if let Some(ctrl_sender) = &self.sender {
                ctrl_sender
                    .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::Delete(
                        peer_id.to_string(),
                    )))
                    .unwrap_or_else(|e| error!("use one: {}", e));
            }
        } else {
            //
            warn!("Trying to remove something which wasn't there ...");
        }
    }
}
