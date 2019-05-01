//! This is a webui about to replace the TUI, to be nice, better accessable, and
//! new technology using websockets

use actix_web::fs; // Todo: during development
use actix_web::{
    actix::{Actor, ActorContext, AsyncContext, StreamHandler},
    http::{self, StatusCode},
    server, ws, App, HttpRequest, HttpResponse, Json, Result,
};
use get_if_addrs;
use hostname;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use uuid::Uuid;

use config;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebServerState {
    uuid: Uuid,
    nr_connections: Arc<Mutex<usize>>,
}

pub struct WebUI {
    uuid: Uuid,
    serve_others: bool,
}

/// needs to be serializable for json
#[derive(Serialize)]
struct JSONResponse {
    cmd: WebCommand,
}

#[derive(Serialize)]
enum WebCommand {
    Started,
    NewMdnsClient(String),
}

/// The formatters here for json output
use std::fmt;
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

impl WebUI {
    pub fn new(id: Uuid, serve: bool) -> Result<Self, ()> {
        let sys = actix::System::new("http-server");
        let connection_count = Arc::new(Mutex::new(0));

        let local_addresses = get_if_addrs::get_if_addrs().unwrap();

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
                    info!("{} is not a loopback device!", ipaddr.addr.ip().to_string());
                    true
                }
            })
            .fold(
                server::HttpServer::new(move || {
                    App::with_state(WebServerState {
                        uuid: id.clone(),
                        nr_connections: connection_count.clone(),
                    })
                    .default_resource(|r| r.f(WebUI::single_page))
                    .resource("/app.js", |r| r.f(WebUI::js_app))
                    //.default_resource(|r| r.f(WebUI::dyn_devel_html)) // Todo: only for devel
                    //.resource("/app.js", |r| r.f(WebUI::dyn_devel_js)) // todo: only for devel
                    .resource("/ws", |r| r.method(http::Method::GET).f(WebUI::ws_index))
                    .resource("/jquery.min.js", |r| {
                        r.f(|_| {
                            HttpResponse::build(StatusCode::OK)
                                .content_type("text/html; charset=utf-8")
                                .body(*config::webui::jquery::JS_JQUERY)
                        })
                    })
                    .resource("favicon.png", |r| {
                        r.f(|_| {
                            HttpResponse::build(StatusCode::OK)
                                .content_type("image/png")
                                .body(*config::webui::FAVICON)
                        })
                    })
                    .resource("sheep.svg", |r| {
                        r.f(|_| {
                            HttpResponse::build(StatusCode::OK)
                                .content_type("image/svg+xml")
                                .body(*config::webui::PIC_SHEEP)
                        })
                    })
                    .resource("/css/{name}", |r| r.f(WebUI::bootstrap_css))
                    .resource("/js/{name}", |r| r.f(WebUI::bootstrap_js))
                }),
                |web_server, ipaddr| {
                    web_server
                        .bind(match ipaddr.addr.ip() {
                            IpAddr::V4(ipv4) => {
                                format!("{}:{:?}", ipv4.to_string(), config::net::PORT_WEBSOCKET)
                            }
                            IpAddr::V6(ipv6) => {
                                format!("{}:{:?}", ipv6.to_string(), config::net::PORT_WEBSOCKET)
                            }
                        })
                        .map(|good_bind| {
                            info!("{} worked!", ipaddr.addr.ip().to_string());
                            good_bind
                        })
                        .map_err(|bad_bind| {
                            error!("{} failed!!", ipaddr.addr.ip().to_string());
                            bad_bind
                        })
                        .unwrap()
                },
            );

        if true {
            web_server.start();
            sys.run();
            Ok(WebUI {
                uuid: id,
                serve_others: serve,
            })
        } else {
            Err(())
        }
    }

    fn dyn_devel_js(_req: &HttpRequest<WebServerState>) -> Result<fs::NamedFile> {
        Ok(fs::NamedFile::open("src/ctrl/webui/js/app.js")?)
    }
    fn dyn_devel_html(_req: &HttpRequest<WebServerState>) -> Result<fs::NamedFile> {
        Ok(fs::NamedFile::open("src/ctrl/webui/html/single_page.html")?)
    }

    fn single_page(req: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        // change state
        let id = req.state().uuid;
        *(req.state().nr_connections.lock().unwrap()) += 1;

        let uuid_page = Self::replace_static_content(*config::webui::HTML_PAGE, &id);

        Ok(HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(uuid_page))
    }

    fn bootstrap_css(req: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        if let Some(css) = req.match_info().get("name") {
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
                "bootstrap-reboot.min.css.map" => {
                    Some(*config::webui::bootstrap::CSS_REBOOT_MIN_MAP)
                }
                _ => {
                    error!("CSS: not found {}", css);
                    None
                }
            };
            if let Some(content) = output {
                Ok(HttpResponse::build(StatusCode::OK)
                    .content_type("text/css; charset=utf-8")
                    .body(content))
            } else {
                Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                    .content_type("text/css; charset=utf-8")
                    .body(""))
            }
        } else {
            Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                .content_type("text/css; charset=utf-8")
                .body(""))
        }
    }

    fn bootstrap_js(req: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        if let Some(js) = req.match_info().get("name") {
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
                Ok(HttpResponse::build(StatusCode::OK)
                    .content_type("application/javascript; charset=utf-8")
                    .body(content))
            } else {
                Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                    .content_type("application/javascript; charset=utf-8")
                    .body(""))
            }
        } else {
            Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                .content_type("application/javascript; charset=utf-8")
                .body(""))
        }
    }

    fn js_app(req: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        let id = req.state().uuid;

        let output = Self::replace_static_content(*config::webui::JS_APP, &id);
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(output))
    }

    fn ws_index(r: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        ws::start(r, MyWebSocket::new())
    }

    fn replace_static_content(html_in: &str, id: &Uuid) -> String {
        // short inline struct
        struct ReplaceStatic<'a> {
            r: &'a str,
            c: String,
        }

        let uuid = id.to_hyphenated().to_string();
        let hostname = hostname::get_hostname().unwrap_or("undefined".to_string());

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
                c: uuid,
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
    type Context = ws::WebsocketContext<Self, WebServerState>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        println!("WS: {:?}", msg);
        match msg {
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

            ctx.ping("");
        });
    }
}
