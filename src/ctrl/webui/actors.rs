///! All actors from webui are represented here
use super::super::super::{
    common::startup::{StartUp, SyncStartUp},
    ctrl::InternalUiMsg,
};
use super::json::{self, WSJson};
// external
use actix::prelude::{StreamHandler, *};
use actix::{Actor, ActorContext, AsyncContext, Context, Handler, Recipient};
use actix_web::web::Json;
use actix_web_actors::ws;
use crossbeam::sync::WaitGroup;
use std::{
    string::String,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
    vec::Vec,
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct MSyncStartup {}

/// Secure ActorSyncStartup by an Option
/// and consume it fast!
pub struct ActorSyncStartup {
    startup_sync: Option<WaitGroup>, // todo: move this to after "start" from browser!!!!!!
}
impl ActorSyncStartup {
    pub fn new(startup_sync: Option<WaitGroup>) -> Self {
        Self { startup_sync }
    }
}
impl Actor for ActorSyncStartup {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("StartUpActor started");
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!("StartUpActor stopped!");
    }
}
impl Handler<MSyncStartup> for ActorSyncStartup {
    type Result = ();

    fn handle(&mut self, msg: MSyncStartup, ctx: &mut Context<Self>) {
        //ctx.stop();
        trace!("webui: waiting ui sync");
        if self.startup_sync.is_some() {
            let a = self.startup_sync.take();
            a.unwrap().wait();
        } else {
            error!("no, this should not used again!");
        }
        trace!("webui: go");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct MRegisterWSClient {
    pub addr: Addr<ActorWebSocket>,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct MServerEvent {
    event: Json<WSJson>,
}

/// Monitors all connected websockets,
/// and therefore distributes the internal incoming
/// messages.
pub struct ActorWSServerMonitor {
    pub receiver: Receiver<InternalUiMsg>,
    pub listeners: Vec<Addr<ActorWebSocket>>, // todo: these are WSCli bla
    pub paths: Vec<String>,
}
impl ActorWSServerMonitor {
    fn register(&mut self, listener: MRegisterWSClient) {
        self.listeners.push(listener.addr);
    }
}

impl Actor for ActorWSServerMonitor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("server monitor got started");

        // send init data after a certain time .... yet poor, timeouts are not a solution
        // todo: waiting is of course not the solution
        let cloned_paths = self.paths.clone();
        ctx.run_later(Duration::from_millis(10000), move |act, _| {
            trace!("sending init!");
            for ws in &act.listeners {
                let answer = Json(json::generate_init_data(&cloned_paths.clone()));
                ws.do_send(MServerEvent { event: answer });
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
                            ws.do_send(MServerEvent {
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
impl Handler<MRegisterWSClient> for ActorWSServerMonitor {
    type Result = ();

    fn handle(&mut self, msg: MRegisterWSClient, _ctx: &mut Context<Self>) {
        info!("something done here");
        self.register(msg);
    }
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
pub struct ActorWebSocket {
    pub starter: Addr<ActorSyncStartup>,
}

impl Actor for ActorWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("ActorWebSocket started");
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
impl Handler<MServerEvent> for ActorWebSocket {
    type Result = ();

    fn handle(&mut self, msg: MServerEvent, ctx: &mut Self::Context) {
        trace!("send: {}", msg.event.to_string());
        ctx.text(msg.event.to_string());
    }
}
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ActorWebSocket {
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
                                    self.starter.do_send(MSyncStartup {})
                                }
                                _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                            }
                        } else {
                            trace!("shittttttty parser");
                            info!("ready from Browser received!");
                            self.starter.do_send(MSyncStartup {})
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
