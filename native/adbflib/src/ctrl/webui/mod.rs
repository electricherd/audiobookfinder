#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
//! This is a webui about to replace the TUI, to be nice, better accessable, and
//! new technology using websockets

mod actors;
mod json;
mod pages;
mod rest_mod;

use super::{
    super::{
        common::{config, paths::SearchPath},
        ctrl::InternalUiMsg,
        net::subs::peer_representation::PeerRepresentation,
    },
    CollectionPathAlive,
};
use actix::{prelude::Addr, Actor};
use actix_web::{
    web::{self, HttpResponse},
    App, Error, HttpRequest, HttpServer,
};
use actix_web_actors::ws;
use actors::{ActorSyncStartup, ActorWSServerMonitor, ActorWebSocket, MRegisterWSClient};
use crossbeam::sync::WaitGroup;
use if_addrs;
use std::{
    io,
    net::IpAddr,
    sync::{mpsc::Receiver, Arc, Mutex},
};

/// Data of the webserver
pub struct WebServerState {
    id: PeerRepresentation,
    nr_connections: Arc<Mutex<usize>>,
    web_port: u16,
}

/// WebUI keep data
pub struct WebUI {
    id: PeerRepresentation,
    paths: Arc<Mutex<SearchPath>>,
    #[allow(dead_code)]
    serve_others: bool, //todo: use it
}

impl WebUI {
    pub fn new(id: PeerRepresentation, serve_others: bool, paths: Arc<Mutex<SearchPath>>) -> Self {
        Self {
            id,
            serve_others,
            paths,
        }
    }

    pub async fn run(
        &self,
        receiver: Receiver<InternalUiMsg>,
        wait_ui_sync: WaitGroup,
        web_port: u16,
    ) -> io::Result<()> {
        let connection_count = Arc::new(Mutex::new(0));
        let path_arc = self.paths.clone();

        let local_addresses = if_addrs::get_if_addrs().unwrap();

        // data
        let initial_state = Arc::new(Mutex::new(WebServerState {
            id: self.id.clone(),
            nr_connections: connection_count.clone(),
            web_port,
        }));

        // very important: after this the actix system, message loop,
        // whatever ... is UP!!!
        let sys = actix::System::new("http-server");

        let web_socket_handler = Arc::new(Mutex::new(
            ActorWSServerMonitor {
                receiver,
                listeners: vec![],
                paths: self.paths.clone(),
            }
            .start(),
        ));

        let address_for_sync_startup =
            ActorSyncStartup::new(wait_ui_sync, web_socket_handler.lock().unwrap().clone()).start();
        let startup_actor_handle = Arc::new(Mutex::new(address_for_sync_startup));

        // count bindings
        let mut nr_bindings: usize = 0;

        // take all local addresses and start if necessary
        // one server with multiple binds
        let web_server = local_addresses
            .into_iter()
            .filter(|ipaddr| {
                let name = ipaddr.name.clone();
                // only use loopback addresses no 2nd ethernet cards
                if ipaddr.addr.is_loopback() {
                    info!(
                        "{} {:?} is a loopback device, good!",
                        ipaddr.addr.ip().to_string(),
                        name
                    );
                    true
                } else {
                    warn!("{} is not a loopback device!", ipaddr.addr.ip().to_string());
                    true
                }
            })
            .fold(
                Ok(HttpServer::new(move || {
                    App::new()
                        // each server has an initial state (e.g. 0 connections)
                        .data(initial_state.clone())
                        .data(web_socket_handler.clone())
                        .data(startup_actor_handle.clone())
                        .data(path_arc.clone())
                        .service(web::resource("/app.js").to(pages::js_app))
                        .default_service(web::resource("").to(pages::single_page))
                        .service(web::resource("peer_page.html").to(|| {
                            HttpResponse::Ok()
                                .content_type("text/html; charset=utf-8")
                                .body(*config::webui::PEER_PAGE)
                        }))
                        .service(web::resource("/jquery.min.js").to(|| {
                            HttpResponse::Ok()
                                .content_type("application/javascript; charset=utf-8")
                                .body(*config::webui::jquery::JS_JQUERY)
                        }))
                        .service(web::resource("/ws_events_dispatcher.js").to(|| {
                            HttpResponse::Ok()
                                .content_type("application/javascript; charset=utf-8")
                                .body(*config::webui::JS_WS_EVENT_DISPATCHER)
                        }))
                        .service(web::resource("favicon.png").to(|| {
                            HttpResponse::Ok()
                                .content_type("image/png")
                                .body(*config::webui::FAVICON)
                        }))
                        .service(web::resource("sheep.svg").to(|| {
                            HttpResponse::Ok()
                                .content_type("image/svg+xml")
                                .body(*config::webui::PIC_SHEEP)
                        }))
                        .service(
                            web::resource("/css/{name}").route(web::get().to(pages::bootstrap_css)),
                        )
                        .service(
                            web::resource("/js/{name}").route(web::get().to(pages::bootstrap_js)),
                        )
                        .service(web::resource("/ws").route(web::get().to(WebUI::websocket_answer)))
                })),
                |web_server_binding_chain: Result<HttpServer<_, _, _, _>, io::Error>, ipaddr| {
                    web_server_binding_chain.and_then(|webserver| {
                        let bind_format = match ipaddr.addr.ip() {
                            IpAddr::V4(ipv4) => format!("{}:{:?}", ipv4.to_string(), web_port),
                            IpAddr::V6(ipv6) => format!("{}:{:?}", ipv6.to_string(), web_port),
                        };
                        let try_bind = webserver.bind(bind_format.clone()).map_err(|error| {
                            error!("binding with ip ({:?}): {:?}", bind_format, error);
                            error
                        })?;
                        nr_bindings += 1;
                        Ok(try_bind)
                    })
                },
            );

        if nr_bindings > 0 {
            web_server
                .map_err(|error| {
                    // clean up
                    actix::System::current().stop();
                    error!("webserver could not start up");
                    error
                })?
                .run();
            sys.run()
        } else {
            // no bindings, no nothing
            error!("No bindings to any http addresses!");
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("no bindings to port {}", &web_port),
            ))
        }
    }

    async fn websocket_answer(
        req: HttpRequest,
        stream: web::Payload,
        data_monitor: web::Data<Arc<Mutex<Addr<ActorWSServerMonitor>>>>,
        data_sync: web::Data<Arc<Mutex<Addr<ActorSyncStartup>>>>,
        data_path: web::Data<Arc<Mutex<SearchPath>>>,
    ) -> Result<HttpResponse, Error> {
        trace!("new websocket answered!");
        let (addr, res) = ws::start_with_addr(
            ActorWebSocket {
                starter: data_sync.lock().unwrap().clone(),
                paths: data_path.get_ref().clone(),
            },
            &req,
            stream,
        )?;
        data_monitor
            .lock()
            .unwrap()
            .do_send(MRegisterWSClient { addr });

        Ok(res)
    }
}
