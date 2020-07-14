//! Taken from dummy behaviour to have a layer of communication which reacts with
//! the embedded state machine (and inner ui), also back to net services:
//! currently kademlia, mdns
//! https://docs.rs/libp2p/0.21.1/libp2p/swarm/struct.DummyBehaviour.html
use libp2p::swarm::{
    protocols_handler, NetworkBehaviour,
    NetworkBehaviourAction::{self, GenerateEvent},
    PollParameters, ProtocolsHandler,
};
use libp2p_core::{
    connection::{ConnectedPoint, ConnectionId},
    Multiaddr, PeerId,
};
use std::{
    collections::vec_deque::VecDeque,
    task::{Context, Poll},
};

use super::{
    sm::{self, AdbfStateChart, Events, Events::*, NewPeerData, States},
    ui_data::UiData,
};

/// Events going from StateMachine back to the net behavior
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SMOutEvents {
    MyPathSearchRunning(bool),
}

//#[derive(Clone, Default)]
pub struct SMBehaviour {
    sm: sm::StateMachine<AdbfStateChart>,
    send_buffer: VecDeque<SMOutEvents>,
}
impl SMBehaviour {
    pub fn new(own_peer: PeerId, ui_data: UiData) -> Self {
        Self {
            sm: AdbfStateChart::init(AdbfStateChart::new(own_peer, ui_data)),
            send_buffer: VecDeque::new(),
        }
    }

    pub fn mdns_new_peer(&mut self, peer_id: &PeerId, multi_addr: &Multiaddr) {
        let new_peer_event = GotANewPeer(NewPeerData {
            id: peer_id.clone(),
            addr: multi_addr.clone(),
        });
        self.process_and_react(new_peer_event);
    }

    pub fn mdns_remove(&mut self, peer_id: &PeerId) {
        let remove_peer_event = HaveToRemovePeer(peer_id.clone());
        self.process_and_react(remove_peer_event);
    }

    fn process_and_react(&mut self, event: Events) {
        let return_state = self.sm.process_event(event);
        match return_state {
            Ok(good_state) => match good_state {
                States::Start => (),                // nothing to do
                States::WaitingForPeerAction => (), // is just waiting
                States::SendKademliaOut => {
                    self.send_buffer
                        .push_back(SMOutEvents::MyPathSearchRunning(true));
                    self.sm.process_event(Done);
                }
            },
            Err(_process_without_valid_state_transition) => (), // this is normal in a state chart
        }
    }
}

impl NetworkBehaviour for SMBehaviour {
    type ProtocolsHandler = protocols_handler::DummyProtocolsHandler;
    type OutEvent = SMOutEvents;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        protocols_handler::DummyProtocolsHandler::default()
    }
    fn addresses_of_peer(&mut self, _: &PeerId) -> Vec<Multiaddr> {
        Vec::new()
    }
    fn inject_connected(&mut self, _: &PeerId) {}
    fn inject_disconnected(&mut self, _: &PeerId) {}
    fn inject_connection_established(&mut self, _: &PeerId, _: &ConnectionId, _: &ConnectedPoint) {}
    fn inject_connection_closed(&mut self, _: &PeerId, _: &ConnectionId, _: &ConnectedPoint) {}

    fn inject_event(
        &mut self,
        _: PeerId,
        _: ConnectionId,
        _: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent,
    ) {
    }

    fn poll(
        &mut self,
        _: &mut Context,
        _: &mut impl PollParameters,
    ) -> Poll<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
            Self::OutEvent,
        >,
    > {
        if let Some(item) = self.send_buffer.pop_front() {
            Poll::Ready(GenerateEvent(item))
        } else {
            Poll::Pending
        }
    }
}
