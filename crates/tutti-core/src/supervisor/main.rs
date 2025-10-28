use tokio::sync::mpsc;
use tutti_types::{Project, ProjectId};

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
    _task: tokio::task::JoinHandle<()>,
    commands_tx: mpsc::Sender<SupervisorCommand>,
}

impl Supervisor {
    pub fn new<P: ProcessManager + Send + Sync + 'static>(
        process_manager: P,
    ) -> (Self, mpsc::Receiver<SupervisorEvent>) {
        let (commands_tx, commands_rx) = mpsc::channel::<SupervisorCommand>(100);
        let (mut inner, output_rx) =
            SupervisorBackground::new(process_manager, commands_tx.clone(), commands_rx);

        let task = tokio::spawn(async move {
            inner.run().await;
        });

        (
            Self {
                _task: task,
                commands_tx,
            },
            output_rx,
        )
    }

    /// Shutdown the supervisor.
    ///
    /// # Errors
    /// Returns an error if the supervisor fails to shutdown.
    pub async fn down(&mut self, project_id: ProjectId) -> Result<()> {
        self.commands_tx
            .send(SupervisorCommand::Down { project_id })
            .await
            .map_err(|err| Error::Internal(err.to_string()))?;

        Ok(())
    }

    /// Shutdown the supervisor.
    ///
    /// # Errors
    /// Returns an error if the supervisor fails to shutdown.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.commands_tx
            .send(SupervisorCommand::Shutdown)
            .await
            .map_err(|err| Error::Internal(err.to_string()))?;

        Ok(())
    }

    /// Start the supervisor.
    ///
    /// # Errors
    /// Returns an error if the supervisor fails to start.
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
            .map_err(|err| Error::Internal(err.to_string()))?;
        self.commands_tx
            .send(SupervisorCommand::Up {
                project_id: project_id.clone(),
                services,
            })
            .await
            .map_err(|err| Error::Internal(err.to_string()))?;

        Ok(())
    }
}
