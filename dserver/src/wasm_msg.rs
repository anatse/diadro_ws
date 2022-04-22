use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WsMessages {
    MousePosition(MousePosition),
    AddFigure(AddFigure),
    AddArrow(AddArrow),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestInfo {
    pub board: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pos2 {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rect {
    min: Pos2,
    max: Pos2,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "mp")]
pub struct MousePosition {
    #[serde(flatten)]
    pub rq: RequestInfo,
    #[serde(rename = "pos")]
    pub position: Pos2,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddFigure {
    pub rq: RequestInfo,
    pub rect: Rect,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddArrow {
    pub rq: RequestInfo,
    pub start_id: String,
    pub end_id: String,
}
