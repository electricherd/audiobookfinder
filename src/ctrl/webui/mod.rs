#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
//! This is a webui about to replace the TUI, to be nice, better accessable, and
//! new technology using websockets
// todo: this file must be logically ordered (websocket, actors, http server, etc.)

mod actors;
mod json;
mod pages;

use super::{
    super::{common::startup::SyncStartUp, config, ctrl::InternalUiMsg},
    CollectionPathAlive, PeerRepresentation,
};
use actors::{ActorSyncStartup, ActorWSServerMonitor, ActorWebSocket, MRegisterWSClient};

// external
use crate::ctrl::webui::actors::MsyncStartup;
use actix::{prelude::Addr, Actor};
use actix_web::{
    web::{self, HttpResponse},
    App, Error, HttpRequest, HttpServer,
};
use actix_web_actors::ws;
use get_if_addrs;
use std::{
    io,
    net::IpAddr,
    string::String,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    vec::Vec,
};

pub struct WebServerState {
    id: PeerRepresentation,
    nr_connections: Arc<Mutex<usize>>,
}

pub struct WebUI {
    id: PeerRepresentation,
    serve_others: bool,
    paths: Vec<String>,
}

impl WebUI {
    pub fn new(id: PeerRepresentation, serve_others: bool, paths: Vec<String>) -> Self {
        Self {
            id,
            serve_others,
            paths,
        }
    }

    pub async fn run(
        &self,
        receiver: Receiver<InternalUiMsg>,
        sync_startup: Sender<SyncStartUp>,
    ) -> io::Result<()> {
        let connection_count = Arc::new(Mutex::new(0));

        let local_addresses = get_if_addrs::get_if_addrs().unwrap();

        // data
        let initial_state = Arc::new(Mutex::new(WebServerState {
            id: self.id.clone(),
            nr_connections: connection_count.clone(),
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

        let sync_startup_actor = Arc::new(Mutex::new(
            ActorSyncStartup {
                startup_sync: sync_startup,
            }
            .start(),
        ));

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
                        .data(sync_startup_actor.clone())
                        .service(web::resource("/app.js").to(pages::js_app))
                        .default_service(web::resource("").to(pages::single_page))
                        //.default_service(web::resource("").to(static_pages::dyn_devel_html)) // Todo: only for devel
                        //.service(web::resource("/app.js").to(static_pages::dyn_devel_js)) // todo: only for devel
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
                            IpAddr::V4(ipv4) => {
                                format!("{}:{:?}", ipv4.to_string(), config::net::PORT_WEBSOCKET)
                            }
                            IpAddr::V6(ipv6) => {
                                format!("{}:{:?}", ipv6.to_string(), config::net::PORT_WEBSOCKET)
                            }
                        };
                        let try_bind = webserver.bind(bind_format.clone()).map_err(|error| {
                            error!("On IP ({:?}): {:?}", bind_format, error);
                            error
                        })?;
                        Ok(try_bind)
                    })
                },
            );
        web_server?.run();
        sys.run()
    }

    async fn websocket_answer(
        req: HttpRequest,
        stream: web::Payload,
        data: web::Data<Arc<Mutex<Addr<ActorWSServerMonitor>>>>,
    ) -> Result<HttpResponse, Error> {
        trace!("new websocket answered!");
        let (addr, res) = ws::start_with_addr(ActorWebSocket::new(), &req, stream)?;
        data.lock().unwrap().do_send(MRegisterWSClient { addr });
        Ok(res)
    }
}
