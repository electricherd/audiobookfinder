//! This is a webui about to replace the TUI, to be nice, better accessable, and
//! new technology using websockets
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{
    error,
    http::{self, StatusCode},
    server, ws, App, Error, HttpRequest, HttpResponse, Result,
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

impl WebUI {
    pub fn new(id: Uuid, serve: bool) -> Result<Self, ()> {
        let sys = actix::System::new("http-server");
        let connection_count = Arc::new(Mutex::new(0));

        let web_server = server::HttpServer::new(move || {
            App::with_state(WebServerState {
                uuid: id.clone(),
                nr_connections: connection_count.clone(),
            })
            .default_resource(|r| r.f(WebUI::single_page))
            .resource("/js/{script_name}.js", |r| r.f(WebUI::static_javascript))
            .resource("/ws", |r| r.method(http::Method::GET).f(WebUI::ws_index))
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

    fn single_page(req: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        // change state
        let id = req.state().uuid;
        *(req.state().nr_connections.lock().unwrap()) += 1;

        let uuid_page = str::replace(
            *config::webui::HTML_PAGE,
            config::webui::HTML_REPLACE_UUID,
            &id.to_hyphenated().to_string(),
        );
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(uuid_page))
    }

    fn static_javascript(req: &HttpRequest<WebServerState>) -> Result<HttpResponse> {
        if let Some(script) = req.match_info().get("script_name") {
            let output = match script {
                "jquery" => (*config::webui::JS_JQUERY).to_string(),
                "app" => str::replace(
                    *config::webui::JS_APP,
                    config::webui::HTML_REPLACE_WEBSOCKET,
                    config::net::WEBSOCKET_ADDR,
                ),
                othername => format!("javascript {} is unknown!", othername),
            };
            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("text/html; charset=utf-8")
                .body(output))
        } else {
            // ToDo: not correct
            Err(error::ErrorBadRequest("bad request"))
        }
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
            ws::Message::Text(text) => ctx.text(text),
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
