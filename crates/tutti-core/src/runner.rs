use futures::StreamExt;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};
use tutti_config::Project;

use crate::{process::BoxStream, CommandSpec, ProcessManager};

async fn follow_output(mut output: BoxStream<Vec<u8>>, rx: Sender<Vec<u8>>) {
    while let Some(line) = output.next().await {
        if rx.send(line).await.is_err() {
            break;
        }
    }
}

#[derive(Debug)]
pub struct Runner<M: ProcessManager> {
    project: Project,
    pm: M,

    tasks: Vec<JoinHandle<()>>,
}

impl<M: ProcessManager> Runner<M> {
    pub fn new(project: Project, pm: M) -> Self {
        let tasks = Vec::with_capacity(project.services.len() * 2);

        Self { project, pm, tasks }
    }

    /// Starts the services defined in the project configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the services fail to start.
    pub async fn up(&mut self) -> anyhow::Result<Receiver<Vec<u8>>> {
        let (tx, rx) = mpsc::channel(10);

        for (name, service) in &self.project.services {
            let service = self
                .pm
                .spawn(CommandSpec {
                    name: name.to_owned(),
                    cmd: service.cmd.clone(),
                    cwd: None,   // TODO
                    env: vec![], // TODO
                })
                .await?;
            let stdout = service.stdout;
            let stderr = service.stderr;

            self.tasks
                .push(tokio::spawn(follow_output(stdout, tx.clone())));
            self.tasks
                .push(tokio::spawn(follow_output(stderr, tx.clone())));
        }

        Ok(rx)
    }

    /// Waits for all services to exit.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the services fail to exit.
    pub async fn wait(&mut self) -> anyhow::Result<()> {
        for task in self.tasks.drain(..) {
            task.await?;
        }

        Ok(())
    }
}
