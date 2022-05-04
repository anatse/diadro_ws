use std::collections::{HashMap, HashSet};

use actix::{Actor, Context, Handler, Recipient};

use crate::{
    messages::{ClientMessage, Connect, Disconnect, Message},
    wasm_msg::{AddArrow, AddFigure, MousePosition},
};

/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct DroServer {
    sessions: HashMap<String, Recipient<Message>>,
    boards: HashMap<String, HashSet<String>>,
}

impl DroServer {
    pub fn new() -> DroServer {
        // default room
        let mut boards = HashMap::new();
        boards.insert("Main".to_owned(), HashSet::new());

        DroServer {
            sessions: HashMap::new(),
            boards,
        }
    }
}

impl DroServer {
    /// Broadcast message to all connected clients, except sender (skip_client)
    fn broadcast(&self, _board: &str, message: &str, skip_client: &str) {
        // tracing::info!("Sessions: {:?}", self.sessions);
        // tracing::info!("Boards: {:?}, board: {}", self.boards, board);
        let _ = self.boards.get("Main").map(|clients| {
            tracing::debug!("{:?}", self.sessions);
            clients
                .iter()
                .filter(|c| *c != skip_client)
                .filter_map(|c| self.sessions.get(c))
                .for_each(|addr| {
                    // tracing::info!("Send message to client: {:?}", addr);
                    let _ = addr.do_send(Message(message.to_owned()));
                })
        });
    }
}

/// Implies actor for Dro server
impl Actor for DroServer {
    type Context = Context<Self>;
}

/// Implies handler for connect message
impl Handler<Connect> for DroServer {
    type Result = String;

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        let id = msg.user_id.to_owned();
        // Just add new user to sessions
        self.sessions.insert(id.clone(), msg.addr);
        self.boards
            .entry("Main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id.clone());
        id
    }
}

impl Handler<Disconnect> for DroServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.user_id);
    }
}

/// Implies handler for client messages such as mouse move for example
impl Handler<ClientMessage> for DroServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!("on client message: {:?}", &msg);
        if !msg.message.is_empty() {
            let (board, user_id) = match &msg.message[0] {
                crate::wasm_msg::WsMessages::MousePosition(MousePosition { rq, .. })
                | crate::wasm_msg::WsMessages::AddArrow(AddArrow { rq, .. })
                | crate::wasm_msg::WsMessages::AddFigure(AddFigure { rq, .. }) => {
                    (rq.board.to_owned(), rq.user.to_owned())
                }
            };

            match serde_json::to_string(&msg.message) {
                Ok(message) => self.broadcast(&board, &message, &user_id),
                Err(err) => tracing::error!("Error serialize: {}", err),
            }
        }
    }
}
