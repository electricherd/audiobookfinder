//! The net module is resonsible for the network related parts,
//! the mDNS registering, mDNS search, ssh server and ssh client.
//! It also let's us startup and perform everything in yet one step.
mod connect_from;
mod connect_to;
mod data;

pub mod key_keeper;

use self::{connect_from::ConnectFromOutside, connect_to::ConnectToOther};
use super::{config, ctrl};
use futures_util::{pin_mut, stream::StreamExt, TryFutureExt};
use libp2p::{
    mdns::{service::MdnsPacket, MdnsService},
    PeerId,
};
use std::borrow::Borrow;
use std::{
    self,
    net::SocketAddr,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Clone)]
enum ToThread<T: Send + Clone> {
    Data(T),
    Stop,
}

/// The Net component keeps controll about everything from net.
pub struct Net {
    peer_id: PeerId,
    addresses_found: Arc<Mutex<Vec<SocketAddr>>>,
    tui_sender: mpsc::Sender<ctrl::SystemMsg>,
    has_tui: bool,
    ssh_handle: thread::JoinHandle<()>,
}

impl Net {
    pub fn new(
        peer_id: PeerId,
        tui: bool,
        sender: mpsc::Sender<ctrl::SystemMsg>,
    ) -> Result<Net, std::io::Error> {
        // the drop of self.dns_handle will unregister
        // so I need to keep it like here :-(
        let net = Net {
            peer_id: peer_id,
            addresses_found: Arc::new(Mutex::new(Vec::new())),
            tui_sender: sender,
            has_tui: tui,
            ssh_handle: thread::spawn(|| {}), // empty join handle
        };
        Ok(net)
    }

    pub fn start_com_server(&mut self) -> Result<(), ()> {
        // delegate this somewhere else
        if let Ok(good_thread) = ConnectFromOutside::create_thread(self.peer_id.clone()) {
            self.ssh_handle = good_thread;
            Ok(())
        } else {
            Err(())
        }
    }

    /// Lookup yet is the start of the networking.
    /// It looks for possible mDNS clients and spawns
    // threads to connect to them.
    // It uses timeouts, checkups.
    pub async fn lookup(&mut self) {
        // we are sending SocketAddr ToThread

        let (sender_ssh_client, receiver_client) = mpsc::channel::<ToThread<SocketAddr>>();
        let sender_ssh_client_stop = sender_ssh_client.clone();
        // I might not need that one here
        // maybe to attach some more data
        //let borrow_arc = &self.addresses_found.clone();
        let has_tui = self.has_tui;
        let peer_id = self.peer_id.clone();

        // to controller messages (mostly tui now)
        let ctrl_sender = self.tui_sender.clone();
        let ctrl_sender2 = self.tui_sender.clone();

        // collection of addresses
        let borrow_arc = self.addresses_found.clone();

        let _get_ip_thread = thread::Builder::new()
            .name("get_ip_thread".to_string())
            .spawn(move || {
                //
                //
                info!("Creating new threads to connecting clients started");
                Self::connect_new_clients(
                    receiver_client,
                    borrow_arc,
                    ctrl_sender,
                    peer_id,
                    has_tui,
                );
            });

        // statistics
        let mut count_valid = 0;
        let mut count_no_cast = 0;

        // prepare everything for mdns thread
        let (mdns_send_ip, mdns_receive_ip) = mpsc::channel::<ToThread<SocketAddr>>();

        let mdns_send_stop = mdns_send_ip.clone();

        // take input from mdns_discoper_thread
        // (new ssh client discovered).
        //
        // If client good send sender_ssh_client to
        // _get_ip_thread
        let _take_mdns_input_thread = thread::Builder::new()
            .name("take_mdns_input_thread".to_string())
            .spawn(move || {
                info!("Processing mdns input threads created!");
                Self::take_mdns_input(
                    mdns_receive_ip,
                    sender_ssh_client,
                    ctrl_sender2,
                    has_tui,
                    // change these
                    &mut count_no_cast,
                    &mut count_valid,
                );
            });

        Self::async_mdns_discover(mdns_send_ip)
            .and_then(|count_response| async move {
                if !has_tui {
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

    async fn async_mdns_discover(
        mdns_send_ip: std::sync::mpsc::Sender<ToThread<SocketAddr>>,
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
        mdns_receive_ip: std::sync::mpsc::Receiver<ToThread<SocketAddr>>,
        sender_ssh_client: std::sync::mpsc::Sender<ToThread<SocketAddr>>,
        ctrl_sender: std::sync::mpsc::Sender<ctrl::SystemMsg>,
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
                .send(ctrl::SystemMsg::StartAnimation(
                    ctrl::Alive::HostSearch,
                    ctrl::Status::OFF,
                ))
                .unwrap();
        }
    }

    fn connect_new_clients(
        receiver_client: std::sync::mpsc::Receiver<ToThread<SocketAddr>>,
        borrow_arc: std::sync::Arc<std::sync::Mutex<std::vec::Vec<SocketAddr>>>,
        ctrl_sender: std::sync::mpsc::Sender<ctrl::SystemMsg>,
        peer_id: PeerId,
        has_tui: bool,
    ) {
        loop {
            let peer_id_clone = peer_id.clone();
            // wait for message
            if let Ok(recv_mesg) = receiver_client.recv() {
                match recv_mesg {
                    ToThread::Data(address) => {
                        //
                        // input address
                        //
                        let try_lock = &mut borrow_arc.lock();
                        if let &mut Ok(ref mut ip_addresses) = try_lock {
                            // search if ip address is already collected
                            if ip_addresses.iter().all(|ip| *ip != address) {
                                // put into collection to not find again
                                ip_addresses.push(address);

                                let count = ip_addresses.len();
                                if has_tui {
                                    ctrl_sender
                                        .send(ctrl::SystemMsg::Update(
                                            ctrl::ReceiveDialog::ShowNewHost,
                                            address.to_string(),
                                        ))
                                        .unwrap();
                                    ctrl_sender
                                        .send(ctrl::SystemMsg::Update(
                                            ctrl::ReceiveDialog::ShowStats {
                                                show: ctrl::NetStats {
                                                    line: count,
                                                    max: 0, //index,
                                                },
                                            },
                                            "".to_string(),
                                        ))
                                        .unwrap();
                                }
                                // create ssh client in new thread
                                let _connect_ip_client_thread = thread::Builder::new()
                                    .name(
                                        ["connect_ip_client_thread_", &address.to_string()]
                                            .concat(),
                                    )
                                    .spawn(move || {
                                        let connector =
                                            ConnectToOther::new(&peer_id_clone, &address);
                                        connector.run();
                                    });
                            }
                        } else {
                            error!("Could not lock ip address list!");
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
    }
}

impl Drop for Net {
    fn drop(&mut self) {
        std::mem::forget(&self.ssh_handle);
        if !self.has_tui {
            println!("Dropping/destroying net");
        }
    }
}
