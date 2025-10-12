use tokio::sync::mpsc;
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
    commands_tx: mpsc::Sender<SupervisorCommand>,
    output_rx: mpsc::Receiver<SupervisorEvent>,
}

impl Supervisor {
    pub async fn new<P: ProcessManager + Send + Sync + 'static>(process_manager: P) -> Self {
        let (commands_tx, commands_rx) = mpsc::channel::<SupervisorCommand>(100);
        let (mut inner, output_rx) =
            SupervisorBackground::new(process_manager, commands_tx.clone(), commands_rx);

        tokio::spawn(async move {
            inner.run().await;
        });

        Self {
            commands_tx,
            output_rx,
        }
    }

    pub fn output(&mut self) -> &mut mpsc::Receiver<SupervisorEvent> {
        &mut self.output_rx
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
