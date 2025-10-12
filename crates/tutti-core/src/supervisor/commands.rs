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
    // EndOfLogs {
    //     project_id: ProjectId,
    //     service: String,
    // },
    // HealthCheck
}
