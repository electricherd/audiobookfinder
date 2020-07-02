//! The net module is resonsible for the network related parts,
//! the mDNS registering, mDNS search, communication server and client.
//! It also let's us startup and perform everything in yet one step.
mod connect_from;
mod connect_to;
mod data;
pub mod key_keeper;

use super::ctrl::{self, ForwardNetMessage};

use async_std::sync::{Arc, Mutex};
use libp2p::{
    mdns::{
        service::{self, MdnsPacket, MdnsPeer},
        MdnsService,
    },
    PeerId,
};
use std::{self, sync::mpsc::Sender, time::Duration};

/// The Net component keeps control about everything from net.
///
/// # Arguments
/// * 'peer_id' - the own peer id (to not talk to much with itself ;-))
/// * 'clients_connected' - All clients that are connected
/// * 'has_ui' - if there is an ui present, that would need update messages///
/// * 'ui_sender' - to send out update ui messages
pub struct Net {
    own_peer_id: PeerId,
    clients_connected: Arc<Mutex<Vec<PeerId>>>,
    has_ui: bool,
    ui_sender: Sender<ctrl::UiUpdateMsg>,
}

impl Net {
    pub fn new(own_peer_id: PeerId, has_ui: bool, ui_sender: Sender<ctrl::UiUpdateMsg>) -> Self {
        Net {
            own_peer_id,
            clients_connected: Arc::new(Mutex::new(Vec::new())),
            has_ui,
            ui_sender,
        }
    }

    /// Lookup yet is the start of the networking.
    /// It looks for possible mDNS clients and spawns eventually
    pub async fn lookup(&mut self) {
        let has_ui = self.has_ui.clone();

        // to controller messages (mostly tui now)
        let ui_update_sender = self.ui_sender.clone();

        // collection of addresses
        let borrow_arc_connected_clients = self.clients_connected.clone();

        // statistics
        let mut count_valid = 0;

        // prepare everything for mdns thread
        let my_peer_id = &self.own_peer_id;
        Self::async_mdns_discover(
            my_peer_id,
            &ui_update_sender,
            has_ui,
            borrow_arc_connected_clients,
            &mut count_valid,
        )
        .await
        .and_then(|count_response| {
            if !has_ui {
                let output_string = format!(
                    "no response from : \n\
                     ----------------- {no_resp:>width$}\n",
                    no_resp = count_response,
                    width = 3
                );
                info!("{}", output_string);
            }
            Ok(())
        })
        .unwrap();
    }

    /// Discovers mdns on the net and should have a whole
    /// process with discovered clients to share data.
    async fn async_mdns_discover(
        own_peer_id: &PeerId,
        ctrl_sender: &std::sync::mpsc::Sender<ctrl::UiUpdateMsg>,
        has_ui: bool,
        borrow_arc_connected_clients: Arc<Mutex<Vec<PeerId>>>,
        count_valid: &mut u32,
    ) -> Result<u32, std::io::Error> {
        let mut count_no_response: u32 = 0;

        let mut service = MdnsService::new()?;
        info!("Started Mdns Service!");

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
        info!("Starting Mdns Looping!");
        // todo: to gracefully stop here, inside the loop could be a receive, which in an
        //       async select! block or just by try_select waits for a terminate message
        //       through a channel.
        loop {
            trace!("taking new package ...");
            let (mut srv, packet) = service.next().await;
            match packet {
                MdnsPacket::Query(query) => {
                    // We detected a libp2p mDNS query on the network. In a real application, you
                    // probably want to answer this query by doing `query.respond(...)`.
                    trace!("Query came in from {:?}", query.remote_addr());
                    if !has_ui {
                        println!("Query came in from: {:?}", query.remote_addr());
                    }

                    // send back own peer ?? todo: a bit more maybe? addresses?
                    let response = service::build_query_response(
                        query.query_id(),
                        own_peer_id.clone(),
                        vec![].into_iter(), // something or leave it empty??
                        Duration::from_secs(120),
                    )
                    .unwrap();
                    srv.enqueue_response(response);
                }
                MdnsPacket::Response(response) => {
                    // We detected a libp2p mDNS response on the network. Responses are for
                    // everyone and not just for the requester, which makes it possible to
                    // passively listen.
                    for new_peer in response.discovered_peers() {
                        // if found myself then don't connect
                        if own_peer_id != new_peer.id() {
                            info!("Discovered: {:?}", new_peer.id());
                            if !has_ui {
                                println!("Discovered: {:?}", new_peer.id());
                            }
                            // These are the self-reported addresses of the peer we just discovered.
                            for addr in new_peer.addresses() {
                                trace!(" Address = {:?}", addr);
                            }

                            *count_valid += 1;
                            Self::connect_new_clients(
                                new_peer,
                                borrow_arc_connected_clients.clone(),
                                ctrl_sender,
                                has_ui,
                            )
                            .await;
                            count_no_response += 1;
                        } else {
                            trace!("Found myself: {:?}", new_peer.id());
                            // to terminal
                            if !has_ui {
                                println!("Found myself: {:?}", new_peer.id());
                            }
                        }
                    }
                }
                MdnsPacket::ServiceDiscovery(query) => {
                    // The last possibility is a service detection query from DNS-SD.
                    // Just like `Query`, in a real application you probably want to call
                    // `query.respond`.
                    info!("Detected service query from {:?}", query.remote_addr());
                }
            }
            // todo: really necessary???
            service = srv
        }
        Ok(count_no_response)
    }

    async fn connect_new_clients(
        new_peer: &MdnsPeer,
        peers_connected: Arc<Mutex<Vec<PeerId>>>,
        ctrl_sender: &std::sync::mpsc::Sender<ctrl::UiUpdateMsg>,
        has_ui: bool,
    ) {
        if let Some(mut guard) = peers_connected.try_lock() {
            let all_stored_peers = &mut *guard;
            // search if ip address is already collected
            if all_stored_peers
                .iter()
                .all(|stored_peer| stored_peer != new_peer.id())
            {
                let count = all_stored_peers.len();
                if has_ui {
                    ctrl_sender
                        .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::new(
                            ctrl::NetMessages::ShowNewHost,
                            new_peer.id().to_string(),
                        )))
                        .unwrap();
                    ctrl_sender
                        .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::new(
                            ctrl::NetMessages::ShowStats {
                                show: ctrl::NetStats {
                                    line: count,
                                    max: 0, //index,
                                },
                            },
                            String::from(""),
                        )))
                        .unwrap();
                }
                // put into collection to not find again
                all_stored_peers.push(new_peer.id().clone());

                // todo: here!!!!!!!!!!!!!!!!!!!!!!!
                // create ssh client in new thread
                // let _connect_ip_client_thread = task::spawn(async move {
                //     let connector = ConnectToOther::new(&id_for_processing);
                //     connector.run();
                // });
            }
        }
        ()
    }
}

impl Drop for Net {
    fn drop(&mut self) {
        if !self.has_ui {
            println!("Dropping/destroying net");
        }
    }
}
