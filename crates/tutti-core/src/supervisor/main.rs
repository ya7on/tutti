use std::collections::{HashMap, HashSet};

use tokio::{sync::mpsc, task::JoinHandle};
use tutti_types::{Project, ProjectId};

use crate::{
    error::{Error, Result},
    process_manager::ProcessManager,
    supervisor::{background::SupervisorBackground, commands::SupervisorCommand},
    Spawned,
};

#[derive(Debug)]
pub enum Status {
    Waiting,
    Starting,
    Running,
    Stopped,
}

#[derive(Debug)]
pub struct RunningService {
    pub name: String,
    pub spawned: Option<Spawned>,
    pub status: Status,
}

#[derive(Debug)]
pub struct Supervisor {
    commands_task: JoinHandle<()>,
    commands_tx: mpsc::Sender<SupervisorCommand>,
}

impl Supervisor {
    pub async fn new<P: ProcessManager + Send + Sync + 'static>(process_manager: P) -> Self {
        let (commands_tx, commands_rx) = mpsc::channel::<SupervisorCommand>(100);
        let mut inner =
            SupervisorBackground::new(process_manager, commands_tx.clone(), commands_rx);

        let commands_task = tokio::spawn(async move {
            inner.run().await;
        });

        Self {
            commands_task,
            commands_tx,
        }
    }

    async fn up(&mut self, project: Project, services: Vec<String>) -> Result<()> {
        tracing::trace!(
            "Received up command for project {project:?} to start services {services:?}"
        );

        let project_id = project.id.clone();

        self.commands_tx
            .send(SupervisorCommand::UpdateConfig {
                project_id: project_id.clone(),
                config: project,
            })
            .await
            .map_err(|err| Error::InternalTransportError(err.to_string()))?;
        self.commands_tx
            .send(SupervisorCommand::Up {
                project_id: project_id.clone(),
                services,
            })
            .await
            .map_err(|err| Error::InternalTransportError(err.to_string()))?;

        Ok(())
    }
}
