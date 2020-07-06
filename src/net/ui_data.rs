/// a very small mod just for ui data send by net. It is important to
/// not send all discovery blindly (e.g. duplicates)
use libp2p_core::PeerId;
use std::{collections::HashMap, sync::mpsc::Sender};

use super::super::ctrl::{self, ForwardNetMessage, UiUpdateMsg};

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
    pub fn register_address(&mut self, peer_id: &PeerId) {
        let ref mut collection = self.ui_shown_peers;
        if collection.get(peer_id).is_none() {
            // add
            collection.insert(peer_id.clone(), ());
            // and send
            if let Some(ctrl_sender) = &self.sender {
                ctrl_sender
                    .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::new(
                        ctrl::NetMessages::ShowNewHost,
                        peer_id.to_string(),
                    )))
                    .unwrap_or_else(|e| error!("use one: {}", e));
                ctrl_sender
                    .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::new(
                        ctrl::NetMessages::ShowStats {
                            show: ctrl::NetStats {
                                line: 0, // todo: some count still
                                max: 0,  //index,
                            },
                        },
                        String::from(""),
                    )))
                    .unwrap();
            }
        }
    }
    pub fn unregister_address(&mut self, peer_id: &PeerId) {
        //
        let ref mut collection = self.ui_shown_peers;
        if collection.remove(peer_id).is_none() {
            //
            warn!("Trying to remove something which wasn't there ...");
        }
    }
}
