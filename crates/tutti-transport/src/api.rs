use serde::{Deserialize, Serialize};
use tutti_types::{Project, ProjectId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TuttiMessage {
    pub id: u32,
    pub req_type: MessageType,
    pub body: TuttiApi,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MessageType {
    Request,
    Response,
    Stream,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TuttiApi {
    Ping,
    Pong,
    Log {
        project_id: ProjectId,
        service: String,
        message: String,
    },
    Up {
        project: Project,
        services: Vec<String>,
    },
    Down {
        project_id: ProjectId,
    },
    Subscribe,
}
