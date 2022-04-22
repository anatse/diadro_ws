use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::{
    messages::{self, ClientMessage, Connect, Disconnect},
    server,
    wasm_msg::WsMessages,
};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsChatSession {
    /// unique session id
    pub id: String,

    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,

    /// joined room
    pub board: String,

    /// peer name
    pub name: Option<String>,

    /// Chat server
    pub addr: Addr<server::DroServer>,
}

impl WsChatSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                tracing::info!("Websocket Client heartbeat failed, disconnecting!");

                // notify server
                act.addr.do_send(Disconnect {
                    user_id: act.id.clone(),
                });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::info!("WS session started: {}", &self.id);

        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(Connect {
                user_id: self.id.clone(),
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        tracing::info!("WS session stopping");
        // notify chat server
        self.addr.do_send(Disconnect {
            user_id: self.id.clone(),
        });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<messages::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: messages::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        tracing::debug!("WEBSOCKET MESSAGE: {:?}", msg);
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
                tracing::debug!("Receive message: {}", m);
                let client_msg = match serde_json::from_str::<Vec<WsMessages>>(m) {
                    Ok(msg) => Some(ClientMessage { message: msg }),
                    Err(err) => {
                        tracing::error!("Error parseing message: {:?}", err);
                        None
                    }
                };

                if let Some(msg) = client_msg {
                    self.addr
                        .send(msg)
                        .into_actor(self)
                        .then(|res, _, _ctx| {
                            match res {
                                Ok(_) => tracing::info!("Ok"),
                                Err(err) => tracing::error!("Something is wrong {}", err),
                            }
                            fut::ready(())
                        })
                        .wait(ctx);
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
