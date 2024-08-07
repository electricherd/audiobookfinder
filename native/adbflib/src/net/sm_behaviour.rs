//! Taken from dummy behaviour to have a layer of communication which reacts with
//! the embedded state machine (and inner ui), also back to net services:
//! currently kademlia, mdns
//! https://docs.rs/libp2p/latest/libp2p/swarm/struct.DummyBehaviour.html
use super::{
    super::data::ipc::{IFCollectionOutputData, IPC},
    sm::{
        self, AdbfStateChart, Error as SMError, Events, Events::*, NewPeerData, States, UpdateData,
    },
    ui_data::UiData,
};
use crate::net::sm_behaviour::protocols_handler::IntoProtocolsHandler;
use crossbeam::channel::Receiver;
use libp2p::{
    core::{
        connection::{ConnectedPoint, ConnectionId},
        Multiaddr, PeerId,
    },
    swarm::{
        protocols_handler, NetworkBehaviour,
        NetworkBehaviourAction::{self, GenerateEvent},
        PollParameters, ProtocolsHandler,
    },
};
use std::{
    collections::vec_deque::VecDeque,
    task::{Context, Poll},
};

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum sm_to_net {
    FoundNewPeer(String),
}

/// Events going from StateMachine back to the net behavior
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug)]
pub enum SMOutEvents {
    ForwardSM(sm_to_net),
    ForwardIPC(IPC),
}

//#[derive(Clone, Default)]
pub struct SMBehaviour {
    sm: sm::StateMachine<AdbfStateChart>,
    own_peer: PeerId,
    send_buffer: VecDeque<SMOutEvents>,
    ipc_receiver: Receiver<IPC>,
}

impl SMBehaviour {
    pub fn new(ipc_receiver: Receiver<IPC>, own_peer: PeerId, ui_data: UiData) -> Self {
        Self {
            sm: AdbfStateChart::init(AdbfStateChart::new(ui_data)),
            own_peer,
            send_buffer: VecDeque::new(),
            ipc_receiver,
        }
    }
    pub fn own_peer(&self) -> PeerId {
        self.own_peer.clone()
    }

    // mdns actions
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

    pub fn update_peer_data(&mut self, peer_id: &PeerId, data: IFCollectionOutputData) {
        let to_update_peer = UpdatePeer(UpdateData {
            id: peer_id.clone(),
            data: data.clone(),
        });
        // todo: this later will be a referenced data (as in SM example on webside)
        self.process_and_react(to_update_peer);
    }

    fn process_and_react(&mut self, event: Events) {
        let return_state = self.sm.process_event(event);
        match return_state {
            Ok(good_state) => match good_state {
                States::Start => (),                // nothing to do
                States::WaitingForPeerAction => (), // is just waiting
            },
            Err(bad_state) => {
                match bad_state {
                    SMError::InvalidEvent => warn!("unexpected event transition"),
                    SMError::GuardFailed => (), // this is quite normal, this is what guards are for
                }
            }
        }
    }
}

/// This is an almost empty SMBehaviour, but callable and with a return OutEvent
/// and a queue that holds the Polling event, and can be influenced. It basically
/// lacks all higher network behaviors, but that was just needed.
/// todo: look how to handle #[behaviour(poll_method = "poll")]
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
    fn inject_connection_established(
        &mut self,
        _: &PeerId,
        _: &ConnectionId,
        _: &ConnectedPoint,
        _: Option<&Vec<Multiaddr>>,
    ) {
    }
    fn inject_connection_closed(
        &mut self,
        _: &PeerId,
        _: &ConnectionId,
        _: &ConnectedPoint,
        _: <<SMBehaviour as NetworkBehaviour>::ProtocolsHandler as IntoProtocolsHandler>::Handler,
    ) {
    }

    fn inject_event(
        &mut self,
        _: PeerId,
        _: ConnectionId,
        _: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent,
    ) {
        // todo ... maybe use inject_event rather than direkt SMBehaviour calls from net_actors?
    }

    fn poll(
        &mut self,
        _: &mut Context,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<SMOutEvents, Self::ProtocolsHandler>> {
        // use this poll for ipc, ipc message will be sent raw for now (not through SM)
        match self.ipc_receiver.try_recv() {
            Ok(ipc_msg) => {
                // todo: maybe filter to which IPC messages go directly to net/kademlia
                //       and which to SM first?
                self.send_buffer.push_back(SMOutEvents::ForwardIPC(ipc_msg))
            }
            Err(_) => (), // just continue
        }
        // and
        if let Some(item) = self.send_buffer.pop_front() {
            Poll::Ready(GenerateEvent(item))
        } else {
            Poll::Pending
        }
    }
}
