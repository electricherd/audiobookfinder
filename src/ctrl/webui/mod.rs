#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
//! This is a webui about to replace the TUI, to be nice, better accessable, and
//! new technology using websockets
// todo: this file must be logically ordered (websocket, actors, http server, etc.)

mod json;
mod pages;

use super::{
    super::{
        common::startup::{StartUp, SyncStartUp},
        config,
        ctrl::InternalUiMsg,
    },
    CollectionPathAlive, PeerRepresentation,
};
use actix::prelude::{StreamHandler, *};
use actix::{Actor, ActorContext, AsyncContext, Context, Handler};
use actix_web::{
    web::{self, HttpResponse, Json},
    App, Error, HttpRequest, HttpServer,
};
use actix_web_actors::ws;
use get_if_addrs;
use json::WSJson;
use std::{
    fmt, io,
    net::IpAddr,
    string::String,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
    vec::Vec,
};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebServerState {
    id: PeerRepresentation,
    nr_connections: Arc<Mutex<usize>>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct RegisterWSClient {
    addr: Addr<MyWebSocket>,
}
#[derive(Message)]
#[rtype(result = "()")]
struct ServerEvent {
    event: Json<WSJson>,
}

/// Monitors all connected websockets,
/// and therefore distributes the internal incoming
/// messages.
struct WSServerMonitor {
    receiver: Receiver<InternalUiMsg>,
    listeners: Vec<Addr<MyWebSocket>>, // todo: these are WSCli bla
    paths: Vec<String>,
    startup_sync: Sender<SyncStartUp>, // todo: move this to after "start" from browser!!!!!!
}

impl Actor for WSServerMonitor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("server monitor got started");

        // send Ready to go to UI -> Main Threads ...
        // todo: find the right place ...!! Be careful, this doesn't work everywhere and leads to
        //       panics!
        ctx.run_later(Duration::from_millis(10), move |act, _| {
            info!("bloooooooooooooooooooooooo");
            StartUp::block_on_sync(act.startup_sync.clone(), "webui");
        });

        // All ctx. result "probably"(?) in an async closures ... hence none of these closures
        // shall not block at all!!

        // send init data after a certain time .... yet poor, timeouts are not a solution
        // todo: waiting is of course not the solution
        let cloned_paths = self.paths.clone();
        ctx.run_later(Duration::from_millis(3000), move |act, _| {
            trace!("sending init!");
            for ws in &act.listeners {
                let answer = Json(json::generate_init_data(&cloned_paths.clone()));
                ws.do_send(ServerEvent { event: answer });
            }
        });

        // todo: this is crap of course, polling in 20ms and try_recv on a receiver
        //       but for now it's fine!!!
        ctx.run_interval(Duration::from_millis(20), |act, _| {
            //loop {

            if let Ok(internal_message) = act.receiver.try_recv() {
                // inform all listeners
                match json::convert_internal_message(&internal_message) {
                    Ok(response_json) => {
                        for ws in &act.listeners {
                            ws.do_send(ServerEvent {
                                event: Json(response_json.clone()),
                            });
                        }
                    }
                    Err(attribute) => {
                        warn!(
                            "No, we don't want the internal variable '{:?}' sent out!",
                            attribute
                        );
                    }
                }
            }
        });
    }
}
impl Handler<RegisterWSClient> for WSServerMonitor {
    type Result = ();

    fn handle(&mut self, msg: RegisterWSClient, _ctx: &mut Context<Self>) {
        info!("something done here");
        self.listeners.push(msg.addr);
    }
}

/// dynamic development files
//#[get("/app.js")]
//fn app_js() -> Result<fs::NamedFile> {
//Ok(fs::NamedFile::open("src/ctrl/webui/js/app.js")?)
//}

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
            WSServerMonitor {
                receiver,
                listeners: vec![],
                paths: self.paths.clone(),
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
                        .data(web_socket_handler.clone())
                        .data(initial_state.clone())
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
        data: web::Data<Arc<Mutex<Addr<WSServerMonitor>>>>,
    ) -> Result<HttpResponse, Error> {
        trace!("new websocket answered!");
        let (addr, res) = ws::start_with_addr(MyWebSocket::new(), &req, stream)?;
        data.lock().unwrap().do_send(RegisterWSClient { addr });
        Ok(res)
    }
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("socket started");
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // On system stop this may or may not run
        warn!("shutting down whole actix-system");
        // todo: a single (!) websocket disconnect shuts down the whole application!!!
        //       in a multi connection program this should be deactivated (e.g. only
        //       per button or never) ... or a certain websocket (localhost ...??) should
        //       be the "master"
        actix::System::current().stop();
    }
}
impl Handler<ServerEvent> for MyWebSocket {
    type Result = ();

    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) {
        trace!("send: {}", msg.event.to_string());
        ctx.text(msg.event.to_string());
    }
}
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    /// Handler for `ws::Message`    
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            Ok(good_message) => {
                trace!("hb handler: {:?}", good_message);
                match good_message {
                    ws::Message::Ping(msg) => {
                        ctx.pong(&msg);
                    }
                    ws::Message::Pong(_) => {}
                    ws::Message::Text(text) => {
                        let m = text.trim();
                        // /command
                        if m.starts_with('/') {
                            let v: Vec<&str> = m.splitn(2, ' ').collect();
                            match v[0] {
                                "/start" => {
                                    info!("ready from Browser received!");
                                }
                                _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                            }
                        }
                    }
                    ws::Message::Binary(bin) => ctx.binary(bin),
                    ws::Message::Close(_) => {
                        ctx.stop();
                    }
                    ws::Message::Nop => (),
                    ws::Message::Continuation(_) => (), // todo: what's this?
                }
            }
            Err(e) => warn!("message was not good {}", e),
        }
    }
}

impl MyWebSocket {
    fn new() -> Self {
        Self {}
    }
}
