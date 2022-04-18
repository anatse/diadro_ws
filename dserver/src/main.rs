use std::collections::HashMap;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use tracing_subscriber::reload::Handle;


#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
}

impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

impl Handler<Message> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

/// Define HTTP actor
struct MyWs {
    pub hb: Instant,
    pub addr: Addr<ChatServer>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

impl MyWs {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(Message("Hello".to_string()));

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => {
                println!("Message::Binary");
                ctx.binary(bin);
            }
            _ => {
                println!("Unknown message");
            }
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs {}, &req, stream);
    // println!("resp: {:?}", resp);
    resp
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

    HttpServer::new(|| 
        App::new()
        .route("/ws/", web::get().to(index)))
        .bind(("0.0.0.0", 8081))?
        .run()
        .await
}