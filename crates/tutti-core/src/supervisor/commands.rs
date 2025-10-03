use tokio::sync::mpsc;

pub type UpResponse = mpsc::Sender<Result<(), ()>>;

pub enum SupervisorCommand {
    Up {
        project_id: tutti_types::ProjectId,
        services: Vec<tutti_types::Service>,
        resp: UpResponse,
    },
}
