use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

use futures::StreamExt;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};
use tutti_config::Project;

use crate::{
    process::{BoxStream, ProcId},
    CommandSpec, ProcessManager,
};

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
    processes: Vec<ProcId>,
}

impl<M: ProcessManager> Runner<M> {
    pub fn new(project: Project, pm: M) -> Self {
        let tasks = Vec::with_capacity(project.services.len() * 2);
        let processes = Vec::with_capacity(project.services.len());

        Self {
            project,
            pm,
            tasks,
            processes,
        }
    }

    /// Performs topological sort on services considering their dependencies.
    /// Returns services in order they should be started.
    ///
    /// # Errors
    ///
    /// Returns an error if a service is not found or if there is a cycle in the dependencies.
    fn topological_sort(&self, service_names: &HashSet<String>) -> anyhow::Result<Vec<String>> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut all_services = HashSet::new();

        let mut to_process = VecDeque::new();
        for name in service_names {
            to_process.push_back(name.clone());
        }

        while let Some(service_name) = to_process.pop_front() {
            if all_services.contains(&service_name) {
                continue;
            }

            let service = self
                .project
                .services
                .get(&service_name)
                .ok_or_else(|| anyhow::anyhow!("Service '{service_name}' not found"))?;

            all_services.insert(service_name.clone());
            graph.entry(service_name.clone()).or_default();
            in_degree.entry(service_name.clone()).or_insert(0);

            for dep in &service.deps {
                if !self.project.services.contains_key(dep) {
                    return Err(anyhow::anyhow!(
                        "Dependency '{dep}' of service '{service_name}' not found",
                    ));
                }

                graph
                    .entry(dep.clone())
                    .or_default()
                    .push(service_name.clone());
                *in_degree.entry(service_name.clone()).or_insert(0) += 1;

                to_process.push_back(dep.clone());
            }
        }

        let mut queue = VecDeque::new();
        for (service, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(service.clone());
            }
        }

        let mut result = Vec::new();
        while let Some(service) = queue.pop_front() {
            result.push(service.clone());

            if let Some(dependents) = graph.get(&service) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        if result.len() != all_services.len() {
            return Err(anyhow::anyhow!("Circular dependency detected"));
        }

        Ok(result)
    }

    /// Starts the services defined in the project configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the services fail to start.
    pub async fn up(&mut self, services: Vec<String>) -> anyhow::Result<Receiver<LogEvent>> {
        let (tx, rx) = mpsc::channel(10);

        let service_names = if services.is_empty() {
            self.project
                .services
                .keys()
                .cloned()
                .collect::<HashSet<_>>()
        } else {
            services
                .into_iter()
                .filter(|name| self.project.services.contains_key(name))
                .collect::<HashSet<_>>()
        };

        let sorted_services = self.topological_sort(&service_names)?;

        let to_run = sorted_services
            .into_iter()
            .map(|name| {
                let service = self
                    .project
                    .services
                    .get(&name)
                    .ok_or_else(|| anyhow::anyhow!("unknown service: {name}"))?;
                Ok((name, service))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        for (name, service) in &to_run {
            let service = self
                .pm
                .spawn(CommandSpec {
                    name: name.to_owned(),
                    cmd: service.cmd.clone(),
                    cwd: service.cwd.clone(),
                    env: service
                        .env
                        .clone()
                        .map(|h| h.into_iter().collect())
                        .unwrap_or_default(),
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
            self.processes.push(service.id);
        }

        Ok(rx)
    }

    /// Stops all services.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the services fail to stop.
    pub async fn down(&mut self) -> anyhow::Result<()> {
        let duration = Duration::from_millis(100);

        for id in self.processes.drain(..) {
            println!("Stopping process {id:?}");
            if self.pm.wait(id, duration).await?.is_some() {
                println!("process {id:?} already stopped");
                continue;
            }

            self.pm.shutdown(id).await?;
            if let Some(exit_code) = self.pm.wait(id, duration).await? {
                println!("process {id:?} stopped with {exit_code} code");
            } else {
                self.pm.kill(id).await?;
                println!("process {id:?} killed");
            }
        }

        Ok(())
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
