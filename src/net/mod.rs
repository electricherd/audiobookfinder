//! The net module is resonsible for the network related parts,
//! the mDNS registering, mDNS search, communication server and client.
//! It also let's us startup and perform everything in yet one step.
mod connect_from;
mod connect_to;
mod data;

pub mod key_keeper;

use self::connect_to::ConnectToOther;
use super::ctrl::{self, ForwardNetMessage};
use async_std::{
    sync::{Arc, Mutex},
    task,
};
use futures_util::TryFutureExt;
use libp2p::{
    mdns::{service::MdnsPacket, MdnsService},
    PeerId,
};
use std::{
    self,
    sync::mpsc::{channel, Sender},
};

#[allow(dead_code)]
#[derive(Clone)]
enum ToThread<T: Send + Clone> {
    Data(T),
    Stop,
}

/// The Net component keeps control about everything from net.
///
/// # Arguments
/// * 'peer_id' - the own peer id (to not talk to much with itself ;-))
/// * 'clients_connected' - All clients that are connected
/// * 'has_ui' - if there is an ui present, that would need update messages///
/// * 'ui_sender' - to send out update ui messages
pub struct Net {
    #[allow(dead_code)]
    peer_id: PeerId,
    clients_connected: Arc<Mutex<Vec<PeerId>>>,
    has_ui: bool,
    ui_sender: Sender<ctrl::UiUpdateMsg>,
}

impl Net {
    pub fn new(peer_id: PeerId, has_ui: bool, ui_sender: Sender<ctrl::UiUpdateMsg>) -> Self {
        Net {
            peer_id,
            clients_connected: Arc::new(Mutex::new(Vec::new())),
            has_ui,
            ui_sender,
        }
    }

    /// Lookup yet is the start of the networking.
    /// It looks for possible mDNS clients and spawns
    pub async fn lookup(&mut self) {
        // threads to connect to them.
        // It uses timeouts, checkups.

        // we are sending SocketAddr ToThread

        let (sender_ssh_client, receiver_client) = channel::<ToThread<PeerId>>();
        let sender_ssh_client_stop = sender_ssh_client.clone();
        // I might not need that one here
        // maybe to attach some more data
        //let borrow_arc = &self.addresses_found.clone();
        let has_ui_mdns_input = self.has_ui.clone();
        let has_ui_stats = self.has_ui.clone();
        let has_ui_new_client = self.has_ui.clone();

        // to controller messages (mostly tui now)
        let ui_update_sender = self.ui_sender.clone();

        // collection of addresses
        let borrow_arc_connected_clients = self.clients_connected.clone();

        let _get_ip_thread = task::spawn(async move {
            //
            //
            info!("Creating new threads to connecting clients started");
            Self::connect_new_clients(
                receiver_client,
                borrow_arc_connected_clients,
                ui_update_sender,
                has_ui_new_client,
            )
            .await;
        });

        // statistics
        let mut count_valid = 0;
        let mut count_no_cast = 0;

        // prepare everything for mdns thread
        let (mdns_send_peer, mdns_receive_peer) = channel::<ToThread<PeerId>>();

        let mdns_send_stop = mdns_send_peer.clone();

        let ctrl_sender = self.ui_sender.clone();
        // take input from mdns_discoper_thread
        // (new ssh client discovered).
        //
        // If client good send sender_ssh_client to
        // _get_ip_thread
        let _take_mdns_input_thread = task::spawn(async move {
            info!("Processing mdns input threads created!");
            Self::take_mdns_input(
                mdns_receive_peer,
                sender_ssh_client,
                ctrl_sender,
                has_ui_mdns_input,
                // change these
                &mut count_no_cast,
                &mut count_valid,
            );
        });

        Self::async_mdns_discover(mdns_send_peer)
            .and_then(|count_response| async move {
                if !has_ui_stats {
                    let output_string = format!(
                        "no response from : {no_resp:>width$}\n\
                         not castable     : {no_cast:>width$}\n",
                        no_resp = count_response,
                        no_cast = count_no_cast,
                        width = 3
                    );
                    info!("{}", output_string);
                }
                Ok(())
            })
            .await
            .unwrap();

        // other thread should not wait for new messages after all
        sender_ssh_client_stop.send(ToThread::Stop).unwrap();
        mdns_send_stop.send(ToThread::Stop).unwrap();
    }

    /// Discovers mdns on the net and should have a whole
    /// process with discovered clients to share data.
    async fn async_mdns_discover(
        _mdns_send_ip: std::sync::mpsc::Sender<ToThread<PeerId>>,
    ) -> Result<usize, std::io::Error> {
        let mut count_no_response: usize = 0;

        let mut service = MdnsService::new()?;
        info!("Started Mdns Service!");

        async move {
            info!("Starting Mdns Looping!");
            loop {
                info!("taking package ...");
                let (srv, packet) = service.next().await;
                match packet {
                    MdnsPacket::Query(query) => {
                        // We detected a libp2p mDNS query on the network. In a real application, you
                        // probably want to answer this query by doing `query.respond(...)`.
                        info!("Detected query from {:?}", query.remote_addr());
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
                            // mdns_send_ip  std::sync::mpsc::Sender<net::ToThread<SocketAddr>>
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

    fn take_mdns_input(
        mdns_receive_ip: std::sync::mpsc::Receiver<ToThread<PeerId>>,
        sender_ssh_client: std::sync::mpsc::Sender<ToThread<PeerId>>,
        ctrl_sender: std::sync::mpsc::Sender<ctrl::UiUpdateMsg>,
        has_tui: bool,
        count_no_cast: &mut usize,
        count_valid: &mut usize,
    ) {
        loop {
            match mdns_receive_ip.recv() {
                Ok(good) => {
                    match good {
                        ToThread::Data(recv_mesg) => {
                            *count_valid += 1;
                            sender_ssh_client
                                .send(ToThread::Data(recv_mesg.clone()))
                                .unwrap();
                        }
                        ToThread::Stop => {
                            break; //break loop
                        }
                    }
                }
                Err(_) => {
                    *count_no_cast += 1;
                    break; //break loop
                }
            }
        } // loop ended
          // finally stop animation
        if has_tui {
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
        receiver_client: std::sync::mpsc::Receiver<ToThread<PeerId>>,
        clients_connected: Arc<Mutex<Vec<PeerId>>>,
        ctrl_sender: std::sync::mpsc::Sender<ctrl::UiUpdateMsg>,
        has_tui: bool,
    ) {
        loop {
            // wait for message
            if let Ok(recv_mesg) = receiver_client.recv() {
                match recv_mesg {
                    ToThread::Data(incoming_id) => {
                        //
                        // input address
                        //
                        let to_push_into_arc_clone = incoming_id.clone();
                        if let Some(mut guard) = clients_connected.try_lock() {
                            let ip_addresses = &mut *guard;
                            // search if ip address is already collected
                            if ip_addresses
                                .iter()
                                .all(|incoming_peer| *incoming_peer != incoming_id)
                            {
                                // put into collection to not find again
                                let id_for_processing = to_push_into_arc_clone.clone();
                                ip_addresses.push(to_push_into_arc_clone);

                                let count = ip_addresses.len();
                                if has_tui {
                                    ctrl_sender
                                        .send(ctrl::UiUpdateMsg::NetUpdate(ForwardNetMessage::new(
                                            ctrl::NetMessages::ShowNewHost,
                                            incoming_id.to_string(),
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
                                let _connect_ip_client_thread = task::spawn(async move {
                                    let connector = ConnectToOther::new(&id_for_processing);
                                    connector.run();
                                });
                            }
                        } else {
                            error!("lock failed!")
                        }
                    }
                    ToThread::Stop => {
                        break; // leave loop and _get_ip_thread
                    }
                }
            } else {
                break; // leave loop and _get_ip_thread
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
