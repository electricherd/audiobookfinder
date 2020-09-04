//! The net module is resonsible for the network related parts,
//! the mDNS registering, mDNS search, communication server and client.
//! It also let's us startup and perform everything in yet one step.
mod behavior;
mod sm;
mod sm_behaviour;
mod storage;
pub mod subs;
mod ui_data;

use self::{sm_behaviour::SMBehaviour, subs::key_keeper, ui_data::UiData};
use super::{ctrl, data::ipc::IPC};
use async_std::task::{self, Context, Poll};
use crossbeam::channel::Receiver;
use futures::prelude::*;
use futures_util::StreamExt;
use libp2p::{
    kad::{record::store::MemoryStore, Kademlia},
    mdns::Mdns,
    PeerId, Swarm,
};
use std::{self, error::Error, sync::mpsc::Sender};

/// The Net component keeps control about everything from net.
///
/// # Arguments
/// * 'has_ui' - if there is an ui present, that would need update messages///
/// * 'ui_sender' - to send out update ui messages
pub struct Net {
    has_ui: bool,
    ui_sender: Sender<ctrl::UiUpdateMsg>,
}

impl Net {
    /// Create a new Net component.    
    pub fn new(has_ui: bool, ui_sender: Sender<ctrl::UiUpdateMsg>) -> Self {
        Net { has_ui, ui_sender }
    }

    /// Lookup yet is the start of the networking. It looks for possible mDNS clients and spawns eventually
    ///
    /// # Arguments
    /// * 'ipc_receiver' - message receiver for internal mass message (called IPC)
    pub async fn lookup(&mut self, ipc_receiver: Receiver<IPC>) {
        let has_ui = self.has_ui.clone();

        // to controller messages (mostly tui now)
        let ui_update_sender = self.ui_sender.clone();

        // prepare everything for mdns thread
        let my_peer_id = key_keeper::get_p2p_server_id();
        Self::build_swarm_and_run(&my_peer_id, &ui_update_sender, ipc_receiver, has_ui)
            .await
            .unwrap();
    }

    /// Discovers mdns on the net and should have a whole
    /// process with discovered clients to share data.
    ///
    /// # Arguments
    /// * 'own_peer_id' - peer id for display mainly
    /// * 'ctrl_sender' - sender for ui update messages
    /// * 'ipc_receiver' - passed on mass message receiver
    /// * 'has_ui' - info on whether usage of ui
    async fn build_swarm_and_run(
        own_peer_id: &PeerId,
        ctrl_sender: &std::sync::mpsc::Sender<ctrl::UiUpdateMsg>,
        ipc_receiver: Receiver<IPC>,
        has_ui: bool,
    ) -> Result<(), Box<dyn Error>> {
        let local_key = &*key_keeper::SERVER_KEY;
        let local_peer_id = PeerId::from(local_key.public());

        // get the transporter
        let transport = behavior::build_noise_transport(
            &*key_keeper::SERVER_KEY,
            Some(*key_keeper::PRESHARED_SECRET),
        );

        let mut swarm = {
            // Create a Kademlia behaviour.
            let store = MemoryStore::new(local_peer_id.clone());
            let kademlia = Kademlia::new(local_peer_id.clone(), store);
            let ui_data = UiData::new(if has_ui {
                Some(ctrl_sender.clone())
            } else {
                None
            });

            let behaviour = behavior::AdbfBehavior {
                kademlia,
                mdns: Mdns::new()?,
                sm_behaviour: SMBehaviour::new(ipc_receiver, own_peer_id.clone(), ui_data),
            };
            Swarm::new(transport, behaviour, local_peer_id.clone())
        };

        // start animation
        if has_ui {
            info!("Start netsearch animation!");
            ctrl_sender
                .send(ctrl::UiUpdateMsg::CollectionUpdate(
                    ctrl::CollectionPathAlive::HostSearch,
                    ctrl::Status::ON,
                ))
                .unwrap();
        }
        // kick off the network actor framework
        Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        let mut listening = false;
        task::block_on(future::poll_fn(move |cx: &mut Context| {
            // this is just future polling for the sake of running swarm
            // and to catch some actions in order for debug messages, NOT MORE
            // because real actions are supposed to be done using the actors!
            loop {
                match swarm.poll_next_unpin(cx) {
                    Poll::Ready(Some(_)) => info!("ha"),
                    Poll::Ready(None) | Poll::Pending => break,
                }
            }
            if !listening {
                for addr in libp2p::Swarm::listeners(&swarm) {
                    info!("Listening on {:?}", addr);
                    listening = true;
                }
            }
            Poll::<()>::Pending
        }));
        Ok(())
    }
}

impl Drop for Net {
    fn drop(&mut self) {
        if !self.has_ui {
            println!("Dropping/destroying net");
        }
    }
}
