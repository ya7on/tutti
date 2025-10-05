use tokio::sync::mpsc;

pub type UpResponse = mpsc::Sender<Result<(), ()>>;

#[derive(Debug)]
pub enum SupervisorCommand {
    Up {
        project: tutti_types::Project,
        services: Vec<String>,
        resp: UpResponse,
    },
}
