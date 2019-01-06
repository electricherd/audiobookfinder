//! This is a webui about to replace the TUI, to be nice, better accessable, and
//! new technology using websockets
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::fs; // Todo: during development
use actix_web::{
    http::{self, Method, StatusCode},
    server, ws, App, HttpRequest, HttpResponse, Json, Result,
};
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

        let web_server = server::HttpServer::new(move || {
            App::with_state(WebServerState {
                uuid: id.clone(),
                nr_connections: connection_count.clone(),
            })
            //.default_resource(|r| r.f(WebUI::single_page))
            //.resource("/js/app.js", |r| r.f(WebUI::js_app))
            .default_resource(|r| r.f(WebUI::dyn_devel_html)) // Todo: only for devel
            .resource("/app.js", |r| r.f(WebUI::dyn_devel_js)) // todo: only for devel
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
            .resource("/fonts/glyphicons-halflings-regular.{name}", |r| {
                r.f(WebUI::bootstrap_fonts)
            })
        })
        .bind(format!("{}", config::net::WEBSOCKET_ADDR));
        if let Ok(configured_server) = web_server {
            configured_server.start();
            sys.run();
            Ok(WebUI {
                uuid: id,
                serve_others: serve,
            })
        } else {
            Err(())
        }
    }

    fn build_default_with_uuid(uuid: &Uuid) -> String {
        str::replace(
            *config::webui::HTML_PAGE,
            config::webui::HTML_REPLACE_UUID,
            &uuid.to_hyphenated().to_string(),
        )
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

        let uuid_page = Self::build_default_with_uuid(&id);

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
                "bootstrap-theme.css" => Some(*config::webui::bootstrap::CSS_THEME),
                "bootstrap-theme.css.map" => Some(*config::webui::bootstrap::CSS_THEME_MAP),
                "bootstrap-theme.min.css" => Some(*config::webui::bootstrap::CSS_THEME_MIN),
                "bootstrap-theme.min.css.map" => Some(*config::webui::bootstrap::CSS_THEME_MIN_MAP),
                _ => {
                    println!("CSS: not found {}", css);
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
                "bootstrap.min.js" => Some(*config::webui::bootstrap::JS),
                "npm.js" => Some(*config::webui::bootstrap::JS_NPM),
                _ => {
                    println!("JS: not found {}", js);
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

    fn bootstrap_fonts(req: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        if let Some(fonts_ext) = req.match_info().get("name") {
            let output = match fonts_ext {
                "eot" => Some(*config::webui::bootstrap::FONT_EOT),
                "woff" => Some(*config::webui::bootstrap::FONT_WOFF),
                "woff2" => Some(*config::webui::bootstrap::FONT_WOFF2),
                "svg" => Some(*config::webui::bootstrap::FONT_SVG),
                _ => {
                    println!("font: not found {}", fonts_ext);
                    None
                }
            };
            if let Some(content) = output {
                Ok(HttpResponse::build(StatusCode::OK)
                    .content_type("application/octet-stream")
                    .body(content))
            } else {
                Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                    .content_type("application/octet-stream")
                    .body(""))
            }
        } else {
            Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                .content_type("application/octet-stream")
                .body(""))
        }
    }

    fn js_app(req: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        let id = req.state().uuid;
        let replace_websocket = str::replace(
            *config::webui::JS_APP,
            config::webui::HTML_REPLACE_WEBSOCKET,
            config::net::WEBSOCKET_ADDR,
        );
        let output = str::replace(
            &replace_websocket,
            config::webui::HTML_REPLACE_UUID,
            &id.to_hyphenated().to_string(),
        );
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(output))
    }

    fn ws_index(r: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        ws::start(r, MyWebSocket::new())
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
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}
