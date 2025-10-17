use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TuttiMessage {
    pub id: u32,
    pub req_type: MessageType,
    pub body: TuttiApi,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TuttiApi {
    Ping,
    Pong,
}
