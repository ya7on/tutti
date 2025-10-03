use tokio::sync::mpsc;

use crate::{
    process_manager::ProcessManager,
    supervisor::commands::{SupervisorCommand, UpResponse},
};

#[derive(Debug)]
pub struct Supervisor<P: ProcessManager> {
    rx: mpsc::Receiver<SupervisorCommand>,
    process_manager: P,
}

impl<P: ProcessManager> Supervisor<P> {
    pub async fn run(&mut self) {
        while let Some(command) = self.rx.recv().await {
            self.handle_command(command).await;
        }
    }

    async fn handle_command(&mut self, command: SupervisorCommand) {
        match command {
            SupervisorCommand::Up {
                project_id,
                services,
                resp,
            } => {
                self.up(project_id, services, resp).await;
            }
        }
    }

    async fn up(
        &mut self,
        project_id: tutti_types::ProjectId,
        services: Vec<tutti_types::Service>,
        resp: UpResponse,
    ) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
