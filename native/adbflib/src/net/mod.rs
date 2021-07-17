//! The net module is resonsible for the network related parts,
//! the mDNS registering, mDNS search, communication server and client.
//! It also let's us startup and perform everything in yet one step.
mod behavior;
mod sm;
mod sm_behaviour;
mod storage;
pub mod subs;
mod ui_data;

use self::{sm_behaviour::SMBehaviour, storage::NetStorage, subs::key_keeper, ui_data::UiData};
use super::{ctrl, data::ipc::IPC};
use async_std::task::{self, Context, Poll};
use crossbeam::channel::{Receiver, Sender};
use futures::prelude::*;
use futures_util::StreamExt;
use libp2p::{
    kad::{record::store::MemoryStore, Kademlia},
    mdns::{Mdns, MdnsConfig},
    PeerId, Swarm,
};
use std::{self, error::Error};

/// The Net component keeps control about everything from net.
pub struct Net {}

impl Net {
    /// Lookup yet is the start of the networking. It looks for possible mDNS clients and spawns eventually
    ///
    /// # Arguments
    /// * 'ui_sender' - to send out update ui messages if ui should be notified
    /// * 'ipc_receiver' - message receiver for internal mass message (called IPC)
    pub async fn lookup(
        &mut self,
        ui_sender: Option<Sender<ctrl::UiUpdateMsg>>,
        ipc_receiver: Receiver<IPC>,
    ) -> Result<(), Box<dyn Error>> {
        // prepare everything for mdns thread
        let my_peer_id = key_keeper::get_p2p_server_id();
        Self::build_swarm_and_run(&my_peer_id, ui_sender, ipc_receiver).await
    }

    /// Discovers mdns on the net and should have a whole
    /// process with discovered clients to share data.
    ///
    /// # Arguments
    /// * 'own_peer_id' - peer id for display mainly
    /// * 'ctrl_sender' - sender for ui update messages if necessary
    /// * 'ipc_receiver' - passed on mass message receiver
    async fn build_swarm_and_run(
        own_peer_id: &PeerId,
        ctrl_sender: Option<Sender<ctrl::UiUpdateMsg>>,
        ipc_receiver: Receiver<IPC>,
    ) -> Result<(), Box<dyn Error>> {
        let local_key = &*key_keeper::SERVER_KEY;
        let local_peer_id = PeerId::from(local_key.public());

        // get the transporter
        let transport = behavior::build_noise_transport(
            &*key_keeper::SERVER_KEY,
            Some(*key_keeper::PRESHARED_SECRET),
        );

        let ui_data = UiData::new(ctrl_sender.clone());

        let mut swarm = {
            // Create a Kademlia behaviour.
            let store = MemoryStore::new(local_peer_id.clone());
            let kademlia = Kademlia::new(local_peer_id.clone(), store);

            let behaviour = behavior::AdbfBehavior {
                kademlia,
                mdns: Mdns::new(MdnsConfig::default()).await?,
                sm_behaviour: SMBehaviour::new(ipc_receiver, own_peer_id.clone(), ui_data),
                storage: NetStorage::new(),
            };
            Swarm::new(transport, behaviour, local_peer_id.clone())
        };

        // start animation
        if let Some(ui_sender) = ctrl_sender {
            info!("Start netsearch animation!");
            ui_sender
                .send(ctrl::UiUpdateMsg::CollectionUpdate(
                    ctrl::CollectionPathAlive::HostSearch,
                    ctrl::Status::ON,
                ))
                .unwrap_or(()); // fixme: for no ui, there should not be Some(ui_sender)
                                //        but for ... -nk it would panic!
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
