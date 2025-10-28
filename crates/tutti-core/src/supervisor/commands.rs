use tokio::sync::mpsc;
use tutti_types::{Project, ProjectId};

pub type UpResponse = mpsc::Sender<Result<(), ()>>;

#[derive(Debug)]
pub enum SupervisorCommand {
    UpdateConfig {
        project_id: ProjectId,
        config: Project,
    },
    Up {
        project_id: ProjectId,
        services: Vec<String>,
    },
    Down {
        project_id: ProjectId,
    },
    EndOfLogs {
        project_id: ProjectId,
        service: String,
    },
    HealthCheckSuccess {
        project_id: ProjectId,
        service: String,
    },
    // HealthCheckFailure {
    //     project_id: ProjectId,
    //     service: String,
    // },
}

#[derive(Debug)]
pub enum SupervisorEvent {
    Log {
        project_id: ProjectId,
        service: String,
        message: String,
    },
    ProjectStopped {
        project_id: ProjectId,
    },
    Error {
        project_id: ProjectId,
        message: String,
    },
}
