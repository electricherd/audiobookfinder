#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
//! This is a webui about to replace the TUI, to be nice, better accessable, and
//! new technology using websockets (since actix changed a lot actix_web, many
//! implementation should probably be reworked)
use super::{
    super::{
        common::startup::{StartUp, SyncStartUp},
        config,
        ctrl::InternalUiMsg,
    },
    PeerRepresentation,
};
use actix::prelude::{StreamHandler, *};
use actix::{Actor, ActorContext, AsyncContext, Context, Handler};
use actix_files as fs;
use actix_web::{
    http::StatusCode,
    web::{self, HttpResponse, Json},
    App, Error, HttpRequest, HttpServer, Responder,
};
use actix_web_actors::ws;
use get_if_addrs;
use hostname;
use std::{
    ffi::OsString,
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
    event: String,
}
struct ServerMonitor {
    listeners: Vec<Addr<MyWebSocket>>,
}

impl Actor for ServerMonitor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("...............server monitor got started");
        ctx.run_interval(Duration::from_secs(5), |act, _| {
            for ws in &act.listeners {
                trace!("...............{}", String::from("Event:"));
                ws.do_send(ServerEvent {
                    event: String::from("Event:"),
                });
            }
        });
    }
}
impl Handler<RegisterWSClient> for ServerMonitor {
    type Result = ();

    fn handle(&mut self, msg: RegisterWSClient, _: &mut Context<Self>) {
        trace!("...............got handle");
        self.listeners.push(msg.addr);
    }
}

/// needs to be serializable for json
#[derive(Serialize)]
struct JSONResponse {
    cmd: WebCommand,
}

#[derive(Serialize)]
enum WebCommand {
    Started,
    #[allow(dead_code)]
    NewMdnsClient(String),
}

impl fmt::Display for JSONResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.cmd)
    }
}
impl fmt::Display for WebCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebCommand::Started => write!(f, "Started"),
            WebCommand::NewMdnsClient(param) => write!(f, "MDNS_IP: {}", param),
        }
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
}

impl WebUI {
    pub fn new(id: PeerRepresentation, serve_others: bool) -> Self {
        WebUI { id, serve_others }
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

        let web_socket_handler = Arc::new(Mutex::new(ServerMonitor { listeners: vec![] }.start()));

        let to_ws_forwarder = async move {
            info!("Trying to poll/receive internal messages");

            loop {
                if let Ok(internal_message) = receiver.recv() {
                    match internal_message {
                        InternalUiMsg::StepAndAnimate(_) => {
                            trace!("StepAndAnimate");
                            //was.send()
                        }
                        InternalUiMsg::StartAnimate(_, _) => {
                            trace!("StartAnimate");
                        }
                        InternalUiMsg::Update(_) => {
                            trace!("Update");
                        }
                    }
                }
            }
            //let sum_result = sum_addr.send(Value(6, 7)).await;
        };
        Arbiter::spawn(to_ws_forwarder);

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
                        .service(web::resource("/app.js").to(WebUI::js_app))
                        .default_service(web::resource("").to(WebUI::single_page))
                        //.default_service(web::resource("").to(WebUI::dyn_devel_html)) // Todo: only for devel
                        //.service(web::resource("/app.js").to(WebUI::dyn_devel_js)) // todo: only for devel
                        .service(web::resource("/jquery.min.js").to(|| {
                            HttpResponse::Ok()
                                .content_type("text/html; charset=utf-8")
                                .body(*config::webui::jquery::JS_JQUERY)
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
                            web::resource("/css/{name}").route(web::get().to(WebUI::bootstrap_css)),
                        )
                        .service(
                            web::resource("/js/{name}").route(web::get().to(WebUI::bootstrap_js)),
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
        StartUp::block_on_sync(sync_startup, "webui");
        web_server?.run();
        sys.run()
    }

    #[allow(dead_code)]
    async fn dyn_devel_html(
        _state: web::Data<Arc<Mutex<WebServerState>>>,
        _req: HttpRequest,
        _path: web::Path<(String,)>,
    ) -> Result<fs::NamedFile, Error> {
        Ok(fs::NamedFile::open("src/ctrl/webui/html/single_page.html")?)
    }

    #[allow(dead_code)]
    async fn dyn_devel_js() -> impl Responder {
        fs::NamedFile::open("src/ctrl/webui/js/app.js")
    }

    async fn single_page(state: web::Data<Arc<Mutex<WebServerState>>>) -> impl Responder {
        // change state
        let mut data = state.lock().unwrap();
        let id = data.id;
        *(data.nr_connections.lock().unwrap()) += 1;

        let id_page = Self::replace_static_content(*config::webui::HTML_PAGE, &id);

        HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(id_page)
    }

    async fn bootstrap_css(path: web::Path<(String,)>) -> impl Responder {
        let css = &*path.0;
        let output = match css {
            "bootstrap.css" => Some(*config::webui::bootstrap::CSS),
            "bootstrap.css.map" => Some(*config::webui::bootstrap::CSS_MAP),
            "bootstrap.min.css" => Some(*config::webui::bootstrap::CSS_MIN),
            "bootstrap.min.css.map" => Some(*config::webui::bootstrap::CSS_MIN_MAP),
            "bootstrap-grid.css" => Some(*config::webui::bootstrap::CSS_GRID),
            "bootstrap-grid.css.map" => Some(*config::webui::bootstrap::CSS_GRID_MAP),
            "bootstrap-grid.min.css" => Some(*config::webui::bootstrap::CSS_GRID_MIN),
            "bootstrap-grid.min.css.map" => Some(*config::webui::bootstrap::CSS_GRID_MIN_MAP),
            "bootstrap-reboot.css" => Some(*config::webui::bootstrap::CSS_REBOOT),
            "bootstrap-reboot.css.map" => Some(*config::webui::bootstrap::CSS_REBOOT_MAP),
            "bootstrap-reboot.min.css" => Some(*config::webui::bootstrap::CSS_REBOOT_MIN),
            "bootstrap-reboot.min.css.map" => Some(*config::webui::bootstrap::CSS_REBOOT_MIN_MAP),
            _ => {
                error!("CSS: not found {}", css);
                None
            }
        };
        if let Some(content) = output {
            HttpResponse::build(StatusCode::OK)
                .content_type("text/css; charset=utf-8")
                .body(content)
        } else {
            HttpResponse::build(StatusCode::NOT_FOUND)
                .content_type("text/css; charset=utf-8")
                .body("")
        }
    }

    async fn bootstrap_js(path: web::Path<(String,)>) -> impl Responder {
        let js = &*path.0;
        let output = match js {
            "bootstrap.js" => Some(*config::webui::bootstrap::JS),
            "bootstrap.js.map" => Some(*config::webui::bootstrap::JS_MAP),
            "bootstrap.min.js" => Some(*config::webui::bootstrap::JS_MIN),
            "bootstrap.min.js.map" => Some(*config::webui::bootstrap::JS_MIN_MAP),
            "bootstrap.bundle.js" => Some(*config::webui::bootstrap::JS_BUNDLE),
            "bootstrap.bundle.js.map" => Some(*config::webui::bootstrap::JS_BUNDLE_MAP),
            "bootstrap.bundle.min.js" => Some(*config::webui::bootstrap::JS_BUNDLE_MIN),
            "bootstrap.bundle.min.js.map" => Some(*config::webui::bootstrap::JS_BUNDLE_MIN_MAP),
            _ => {
                error!("JS: not found {}", js);
                None
            }
        };
        if let Some(content) = output {
            HttpResponse::build(StatusCode::OK)
                .content_type("application/javascript; charset=utf-8")
                .body(content)
        } else {
            HttpResponse::build(StatusCode::NOT_FOUND)
                .content_type("application/javascript; charset=utf-8")
                .body("")
        }
    }

    async fn js_app(state: web::Data<Arc<Mutex<WebServerState>>>) -> impl Responder {
        let data = state.lock().unwrap();
        let id = data.id;

        let output = Self::replace_static_content(*config::webui::JS_APP, &id);
        HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(output)
    }

    async fn websocket_answer(
        req: HttpRequest,
        stream: web::Payload,
        data: web::Data<Arc<Mutex<Addr<ServerMonitor>>>>,
    ) -> Result<HttpResponse, Error> {
        let (addr, res) = ws::start_with_addr(MyWebSocket::new(), &req, stream)?;
        trace!("........ websocket start with address");
        data.lock().unwrap().do_send(RegisterWSClient { addr });
        trace!("........ websocket send");
        Ok(res)
    }

    fn replace_static_content(html_in: &str, id: &PeerRepresentation) -> String {
        // short inline struct
        struct ReplaceStatic<'a> {
            r: &'a str,
            c: String,
        }

        let id_string = std::format!("{:x?}", id);
        let hostname = hostname::get()
            .unwrap_or(OsString::from("undefined"))
            .into_string()
            .unwrap_or(String::from("undefined"));

        let changers: [ReplaceStatic; 4] = [
            ReplaceStatic {
                r: config::net::HTML_REPLACE_STATIC_URL_SOURCE,
                c: config::net::HTML_URL_SOURCE.to_string(),
            },
            ReplaceStatic {
                r: config::webui::HTML_REPLACE_WEBSOCKET,
                c: config::net::WEBSOCKET_ADDR.to_string(),
            },
            ReplaceStatic {
                r: config::webui::HTML_REPLACE_UUID,
                c: id_string,
            },
            ReplaceStatic {
                r: config::webui::HTML_REPLACE_HOSTNAME,
                c: hostname,
            },
        ];
        let mut replace_this = html_in.to_string();

        for replacer in &changers {
            replace_this = str::replace(&replace_this, &replacer.r, &replacer.c);
        }
        replace_this
    }
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("heartbeat started");
        self.hb(ctx);
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // On system stop this may or may not run
        warn!("shutting down whole actix-system");
        actix::System::current().stop();
    }
}
impl Handler<ServerEvent> for MyWebSocket {
    type Result = ();

    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) {
        trace!("goooooooooo");
        ctx.text(msg.event);
    }
}
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    /// Handler for `ws::Message`    
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            Ok(good_message) => {
                trace!(":: {:?}", good_message);
                match good_message {
                    ws::Message::Ping(msg) => {
                        self.hb = Instant::now();
                        ctx.pong(&msg);
                    }
                    ws::Message::Pong(_) => {
                        self.hb = Instant::now();
                    }
                    ws::Message::Text(text) => {
                        let m = text.trim();
                        // /command
                        if m.starts_with('/') {
                            let v: Vec<&str> = m.splitn(2, ' ').collect();
                            match v[0] {
                                "/start" => {
                                    ctx.text(
                                        Json(JSONResponse {
                                            cmd: WebCommand::Started,
                                        })
                                        .to_string(),
                                    );
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
        Self { hb: Instant::now() }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                error!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}
