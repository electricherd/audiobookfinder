///! All actors from webui are represented here
use super::super::super::{
    common::startup::{StartUp, SyncStartUp},
    ctrl::InternalUiMsg,
};
use super::json::{self, WSJson};

use actix::prelude::{StreamHandler, *};
use actix::{Actor, ActorContext, AsyncContext, Context, Handler};
use actix_web::web::Json;
use actix_web_actors::ws;
use std::{
    string::String,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
    vec::Vec,
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct MsyncStartup {}

pub struct StartupActor {
    pub startup_sync: Sender<SyncStartUp>, // todo: move this to after "start" from browser!!!!!!
}
impl Actor for StartupActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("StartUpActor started");
    }
}
impl Handler<MsyncStartup> for StartupActor {
    type Result = ();

    fn handle(&mut self, msg: MsyncStartup, _ctx: &mut Context<Self>) {
        info!("MsyncStartup received");
        StartUp::block_on_sync(self.startup_sync.clone(), "webui");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterWSClient {
    pub addr: Addr<MyWebSocket>,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerEvent {
    event: Json<WSJson>,
}

/// Monitors all connected websockets,
/// and therefore distributes the internal incoming
/// messages.
pub struct WSServerMonitor {
    pub receiver: Receiver<InternalUiMsg>,
    pub listeners: Vec<Addr<MyWebSocket>>, // todo: these are WSCli bla
    pub paths: Vec<String>,
    pub startup_sync: Sender<SyncStartUp>,
}
impl WSServerMonitor {
    fn register(&mut self, listener: RegisterWSClient) {
        self.listeners.push(listener.addr);
    }
}

impl Actor for WSServerMonitor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("server monitor got started");

        StartUp::block_on_sync(self.startup_sync.clone(), "webui");

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
        self.register(msg);
    }
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
pub struct MyWebSocket {}

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
    pub fn new() -> Self {
        Self {}
    }
}
