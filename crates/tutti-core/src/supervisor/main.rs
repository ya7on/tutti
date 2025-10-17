use tokio::{
    select,
    sync::mpsc::{self, Receiver},
};
use tutti_types::Project;

use crate::{
    error::{Error, Result},
    process_manager::ProcessManager,
    supervisor::{
        background::SupervisorBackground,
        commands::{SupervisorCommand, SupervisorEvent},
    },
};

#[derive(Debug)]
pub struct Supervisor {
    task: tokio::task::JoinHandle<()>,
    commands_tx: mpsc::Sender<SupervisorCommand>,
}

impl Supervisor {
    pub async fn new<P: ProcessManager + Send + Sync + 'static>(
        process_manager: P,
    ) -> (Self, mpsc::Receiver<SupervisorEvent>) {
        let (commands_tx, commands_rx) = mpsc::channel::<SupervisorCommand>(100);
        let (mut inner, output_rx) =
            SupervisorBackground::new(process_manager, commands_tx.clone(), commands_rx);

        let task = tokio::spawn(async move {
            inner.run().await;
        });

        (Self { task, commands_tx }, output_rx)
    }

    pub async fn down(&mut self, project: Project) -> Result<()> {
        self.commands_tx
            .send(SupervisorCommand::Down {
                project_id: project.id,
            })
            .await
            .map_err(|err| Error::InternalTransportError(err.to_string()))?;

        Ok(())
    }

    pub async fn up(&mut self, project: Project, services: Vec<String>) -> Result<()> {
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
