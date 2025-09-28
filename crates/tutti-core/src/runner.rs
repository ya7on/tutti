use futures::StreamExt;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};
use tutti_config::Project;

use crate::{process::BoxStream, CommandSpec, ProcessManager};

#[derive(Debug)]
pub enum LogEvent {
    Log { service_name: String, line: Vec<u8> },
    Stop { service_name: String },
}

async fn follow_output(
    is_stdout: bool,
    mut output: BoxStream<Vec<u8>>,
    service_name: String,
    rx: Sender<LogEvent>,
) {
    while let Some(line) = output.next().await {
        if rx
            .send(LogEvent::Log {
                service_name: service_name.clone(),
                line,
            })
            .await
            .is_err()
        {
            break;
        }
    }
    if is_stdout && rx.send(LogEvent::Stop { service_name }).await.is_err() {
        eprintln!("Failed to send stop event");
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
    pub async fn up(&mut self, services: Vec<String>) -> anyhow::Result<Receiver<LogEvent>> {
        let (tx, rx) = mpsc::channel(10);

        for (name, service) in &self.project.services {
            if !services.contains(name) && !services.is_empty() {
                continue;
            }

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

            self.tasks.push(tokio::spawn(follow_output(
                true,
                stdout,
                name.clone(),
                tx.clone(),
            )));
            self.tasks.push(tokio::spawn(follow_output(
                false,
                stderr,
                name.clone(),
                tx.clone(),
            )));
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
