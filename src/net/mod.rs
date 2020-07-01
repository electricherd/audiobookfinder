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
        let mut count_no_cast = 0;

        // prepare everything for mdns thread
        let my_peer_id = &self.own_peer_id;
        Self::async_mdns_discover(
            my_peer_id,
            &ui_update_sender,
            has_ui,
            borrow_arc_connected_clients,
            &mut count_no_cast,
            &mut count_valid,
        )
        .await
        .and_then(|count_response| {
            if !has_ui {
                let output_string = format!(
                    "no response from : {no_resp:>width$}\n\
                     no cast:           {no_cast:>width$}\n",
                    no_resp = count_response,
                    no_cast = count_no_cast,
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
        my_peer_id: &PeerId,
        ctrl_sender: &std::sync::mpsc::Sender<ctrl::UiUpdateMsg>,
        has_ui: bool,
        borrow_arc_connected_clients: Arc<Mutex<Vec<PeerId>>>,
        count_no_cast: &mut u32,
        count_valid: &mut u32,
    ) -> Result<u32, std::io::Error> {
        let mut count_no_response: u32 = 0;

        let mut service = MdnsService::new()?;
        info!("Started Mdns Service!");

        async move {
            info!("Starting Mdns Looping!");

            loop {
                trace!("taking package ...");
                let (mut srv, packet) = service.next().await;
                match packet {
                    MdnsPacket::Query(query) => {
                        // We detected a libp2p mDNS query on the network. In a real application, you
                        // probably want to answer this query by doing `query.respond(...)`.
                        trace!("Detected query from {:?}", query.remote_addr());
                        let response = service::build_query_response(
                            query.query_id(),
                            my_peer_id.clone(),
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
                        for peer in response.discovered_peers() {
                            info!("Discovered peer {:?}", peer.id());
                            // These are the self-reported addresses of the peer we just discovered.
                            for addr in peer.addresses() {
                                println!(" Address = {:?}", addr);
                            }
                            // todo: filter own address already here
                            Self::take_mdns_input(
                                peer,
                                ctrl_sender,
                                has_ui,
                                borrow_arc_connected_clients.clone(),
                                count_no_cast,
                                count_valid,
                            )
                            .await;
                            count_no_response += 1;
                        }
                    }
                    MdnsPacket::ServiceDiscovery(query) => {
                        // The last possibility is a service detection query from DNS-SD.
                        // Just like `Query`, in a real application you probably want to call
                        // `query.respond`.
                        info!("Detected service query from {:?}", query.remote_addr());
                    }
                }
                service = srv
            }
        }
        .await;
        Ok(count_no_response)
    }

    async fn take_mdns_input(
        mdns_receive_ip: &MdnsPeer,
        ctrl_sender: &std::sync::mpsc::Sender<ctrl::UiUpdateMsg>,
        has_ui: bool,
        borrow_arc_connected_clients: Arc<Mutex<Vec<PeerId>>>,
        count_no_cast: &mut u32,
        count_valid: &mut u32,
    ) {
        *count_valid += 1;
        Self::connect_new_clients(
            mdns_receive_ip,
            borrow_arc_connected_clients,
            ctrl_sender,
            has_ui,
        )
        .await;

        // finally stop animation
        if has_ui {
            info!("Stop animation!");
            ctrl_sender
                .send(ctrl::UiUpdateMsg::CollectionUpdate(
                    ctrl::CollectionPathAlive::HostSearch,
                    ctrl::Status::OFF,
                ))
                .unwrap();
        }
    }

    async fn connect_new_clients(
        receiver_client: &MdnsPeer,
        clients_connected: Arc<Mutex<Vec<PeerId>>>,
        ctrl_sender: &std::sync::mpsc::Sender<ctrl::UiUpdateMsg>,
        has_ui: bool,
    ) {
        // wait for message
        //
        // input address
        //
        //let to_push_into_arc_clone = incoming_id.clone();
        if let Some(mut guard) = clients_connected.try_lock() {
            let ip_addresses = &mut *guard;
            // search if ip address is already collected
            if ip_addresses
                .iter()
                .all(|stored_peer_ids| stored_peer_ids != receiver_client.id())
            {
                // put into collection to not find again
                // todo: what this
                // let id_for_processing = to_push_into_arc_clone.clone();
                // ip_addresses.push(to_push_into_arc_clone);

                let count = ip_addresses.len();
                if has_ui {
                    ctrl_sender
                        .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::new(
                            ctrl::NetMessages::ShowNewHost,
                            receiver_client.id().to_string(),
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
