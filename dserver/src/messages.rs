use actix::prelude::*;

use crate::wasm_msg::WsMessages;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications
/// New chat session is created
#[derive(Message)]
#[rtype(String)]
pub struct Connect {
    pub user_id: String,
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: String,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub message: Vec<WsMessages>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ListRooms;
