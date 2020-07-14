//! The StateChart for the server ...
//! StateMachine type/struct is created inside here by macros I suppose, so using it
//! needs to "reimport" this file/mod
use crate::net::ui_data::UiData;
use libp2p::core::{Multiaddr, PeerId};
use smlang::statemachine;

#[derive(PartialEq)]
pub struct Go {}
#[derive(PartialEq)]
pub struct StartData {
    pub own_address: PeerId,
}
#[derive(PartialEq)]
pub struct NewPeerData {
    pub id: PeerId,
    pub addr: Multiaddr,
}

statemachine! {
    *Start + Go = WaitingForPeerAction,
    WaitingForPeerAction + GotANewPeer(NewPeerData) [ not_known ] / process_new_peer = SendKademliaOut,
    SendKademliaOut + Done = WaitingForPeerAction,
    WaitingForPeerAction + HaveToRemovePeer(PeerId) [ known ] / remove_peer = WaitingForPeerAction,
}

pub struct AdbfStateChart {
    own_peer: PeerId,
    ui_data: UiData,
}
impl AdbfStateChart {
    pub fn new(own_peer: PeerId, ui_data: UiData) -> Self {
        Self { own_peer, ui_data }
    }
}
impl AdbfStateChart {
    pub fn init(me: Self) -> StateMachine<AdbfStateChart> {
        let mut sm = StateMachine::new(me);
        // todo: is that ok, start here?
        if sm.process_event(Events::Go).is_err() {
            error!("No, no, re-check state chart to hold the Go initial state!");
        }
        sm
    }
}

impl StateMachineContext for AdbfStateChart {
    // guards
    fn not_known(&mut self, event_data: &NewPeerData) -> bool {
        !self.ui_data.has_peer(&event_data.id)
    }
    fn known(&mut self, event_data: &PeerId) -> bool {
        self.ui_data.has_peer(&event_data)
    }

    // actions
    fn process_new_peer(&mut self, peer_data: &NewPeerData) {
        let ref peer_id = peer_data.id;
        let multiaddr = peer_data.addr.clone();
        self.ui_data.register_address(peer_id, &multiaddr);
    }

    fn remove_peer(&mut self, peer_id: &PeerId) {
        info!("got {}", peer_id);
        self.ui_data.unregister_address(&peer_id);
    }
}
