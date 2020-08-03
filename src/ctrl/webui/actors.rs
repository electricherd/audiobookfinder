///! All actors from webui are represented here
use super::{
    super::super::ctrl::InternalUiMsg,
    json::{self, WSJsonIn, WSJsonOut},
};
use actix::{
    prelude::{StreamHandler, *},
    Actor, ActorContext, AsyncContext, Context, Handler,
};
use actix_web::web::Json;
use actix_web_actors::ws;
use crossbeam::sync::WaitGroup;
use std::{string::String, sync::mpsc::Receiver, time::Duration, vec::Vec};

#[derive(Message)]
#[rtype(result = "()")]
pub struct MSyncStartup {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct MDoneSyncStartup {}

/// Secure ActorSyncStartup by an Option
/// and consume it fast!
pub struct ActorSyncStartup {
    startup_sync: Option<WaitGroup>,
    inform_to: Addr<ActorWSServerMonitor>,
}
impl ActorSyncStartup {
    pub fn new(startup_sync: Option<WaitGroup>, inform_to: Addr<ActorWSServerMonitor>) -> Self {
        Self {
            startup_sync,
            inform_to,
        }
    }
}
impl Actor for ActorSyncStartup {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        trace!("StartUpActor started");
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!("StartUpActor stopped!");
    }
}
impl Handler<MSyncStartup> for ActorSyncStartup {
    type Result = ();

    fn handle(&mut self, _msg: MSyncStartup, _ctx: &mut Context<Self>) {
        //ctx.stop();
        if self.startup_sync.is_some() {
            let a = self.startup_sync.take();
            trace!("webui: waiting ui sync");
            a.unwrap().wait();
            trace!("webui: go");
            self.inform_to.do_send(MDoneSyncStartup {});
        } else {
            error!("no, this should not used again!");
        }
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
    event: Json<WSJsonOut>,
}

/// Monitors all connected websockets,
/// and therefore distributes the internal incoming
/// messages.
pub struct ActorWSServerMonitor {
    pub receiver: Receiver<InternalUiMsg>,
    pub listeners: Vec<Addr<ActorWebSocket>>,
    pub paths: Vec<String>,
}
impl ActorWSServerMonitor {
    fn register(&mut self, listener: MRegisterWSClient) {
        trace!("register new listener");
        self.listeners.push(listener.addr);
    }
}

impl Actor for ActorWSServerMonitor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("server monitor got started");

        // when failing during startup this is important
        // todo: this is crap of course, polling in 20ms and try_recv on a receiver
        //       but for now it's fine!!! look at general Poll::, since libp2p uses
        //       just like actix here tokio, and Polling is used there!
        // fixme: part 2 ... cannot use actix functions to prevent run_interval to
        //        run even if actix is not correctly running, fix is at webui to
        //        not start ActorWSServerMonitor when there is no running http server
        //        (this happens when binding fails ... but it is already too late then)
        ctx.run_interval(Duration::from_millis(20), |act, _| {
            if let Ok(internal_message) = act.receiver.try_recv() {
                match internal_message {
                    InternalUiMsg::Terminate => actix::System::current().stop(),
                    _ => {
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
                }
            }
        });
    }
}
impl Handler<MRegisterWSClient> for ActorWSServerMonitor {
    type Result = ();

    fn handle(&mut self, msg: MRegisterWSClient, _ctx: &mut Context<Self>) {
        self.register(msg);
    }
}

impl Handler<MDoneSyncStartup> for ActorWSServerMonitor {
    type Result = ();

    fn handle(&mut self, _msg: MDoneSyncStartup, _ctx: &mut Context<Self>) {
        trace!("sending init!");
        // todo: init shall be sent of course for each(!) new connecting websocket
        //       and this is going to ALL not matter if they did receive already.
        //       It can be a browser issues, since browser security prevents a lot of
        //       things, even to start cross-side javascript connection.
        let cloned_paths = self.paths.clone();
        for ws in &self.listeners {
            let answer = Json(json::generate_init_data(&cloned_paths.clone()));
            ws.do_send(MServerEvent { event: answer });
        }
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
    fn started(&mut self, _ctx: &mut Self::Context) {
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
                match good_message {
                    ws::Message::Ping(msg) => {
                        ctx.pong(&msg);
                    }
                    ws::Message::Pong(_) => {}
                    ws::Message::Text(text) => {
                        let m = text.trim();
                        // /command
                        match json::convert_external_message(m) {
                            Ok(incoming) => match incoming {
                                WSJsonIn::start => {
                                    trace!("ready from Browser received!");
                                    self.starter.do_send(MSyncStartup {})
                                }
                            },
                            Err(wrong_message) => {
                                error!("received wrong message: {}", wrong_message);
                            }
                        }
                    }
                    ws::Message::Binary(bin) => ctx.binary(bin),
                    ws::Message::Close(_) => {
                        ctx.stop();
                    }
                    ws::Message::Nop => (),
                    ws::Message::Continuation(_) => (),
                }
            }
            Err(e) => warn!("message was not good {}", e),
        }
    }
}
