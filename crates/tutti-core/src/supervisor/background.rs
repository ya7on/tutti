use std::collections::{HashMap, HashSet, VecDeque};

use futures::StreamExt;
use tutti_types::{Project, ProjectId, Service};

use crate::{
    error::{Error, Result},
    supervisor::{commands::SupervisorEvent, SupervisorCommand},
    CommandSpec, ProcId, ProcessManager,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Waiting { wait_for: Vec<String> },
    Starting,
    Running,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct RunningService {
    pub name: String,
    pub pid: Option<ProcId>,
    pub status: Status,
}

#[derive(Debug)]
pub struct SupervisorBackground<P: ProcessManager> {
    process_manager: P,
    storage: HashMap<ProjectId, Vec<RunningService>>,
    config: HashMap<ProjectId, Project>,

    commands_tx: tokio::sync::mpsc::Sender<SupervisorCommand>,
    commands_rx: tokio::sync::mpsc::Receiver<SupervisorCommand>,

    output_tx: tokio::sync::mpsc::Sender<SupervisorEvent>,
}

impl<P: ProcessManager> SupervisorBackground<P> {
    pub fn new(
        process_manager: P,
        commands_tx: tokio::sync::mpsc::Sender<SupervisorCommand>,
        commands_rx: tokio::sync::mpsc::Receiver<SupervisorCommand>,
    ) -> (Self, tokio::sync::mpsc::Receiver<SupervisorEvent>) {
        tracing::info!("SupervisorBackground initialized");

        let (output_tx, output_rx) = tokio::sync::mpsc::channel(100);
        (
            Self {
                process_manager,
                storage: HashMap::new(),
                config: HashMap::new(),
                commands_tx,
                commands_rx,
                output_tx,
            },
            output_rx,
        )
    }

    pub async fn run(&mut self) {
        tracing::info!("SupervisorBackground started");

        while let Some(command) = self.commands_rx.recv().await {
            tracing::debug!("Received command: {:?}", command);

            if let Err(err) = self.handle_commands(command).await {
                tracing::error!("Error handling command: {err:?}");
            }
        }
    }

    async fn handle_commands(&mut self, command: SupervisorCommand) -> Result<()> {
        tracing::debug!("Handling command: {:?}", command);

        match command {
            SupervisorCommand::UpdateConfig { project_id, config } => {
                tracing::debug!("Updating config for project {project_id:?}");

                self.update_config(project_id, config);
                Ok(())
            }
            SupervisorCommand::Up {
                project_id,
                services,
            } => {
                tracing::debug!(
                    "Starting services for project {project_id:?} with services: {services:?}",
                );

                self.up(project_id, services).await?;
                Ok(())
            }
            SupervisorCommand::Down { project_id } => {
                tracing::debug!("Stopping services for project {project_id:?}");

                self.down(project_id).await?;
                Ok(())
            }
            SupervisorCommand::EndOfLogs {
                project_id,
                service,
            } => {
                tracing::debug!(
                    "Getting end of logs for project {project_id:?} and service {service:?}"
                );

                self.end_of_logs(project_id, service)?;
                Ok(())
            }
            SupervisorCommand::HealthCheckSuccess {
                project_id,
                service,
            } => {
                tracing::debug!(
                    "Health check success for project {project_id:?} and service {service:?}"
                );

                self.health_check_success(project_id, service).await?;
                Ok(())
            }
        }
    }

    fn update_config(&mut self, project_id: ProjectId, new_config: Project) {
        tracing::info!("Updating config for project {project_id:?}");

        self.config.insert(project_id, new_config);
    }

    async fn up(&mut self, project_id: ProjectId, services: Vec<String>) -> Result<()> {
        tracing::info!("Starting {services:?} services for project {project_id:?}");

        let Some(config) = self.config.get(&project_id).cloned() else {
            return Err(Error::ProjectNotFound(project_id));
        };

        let services = Self::toposort(&config, &services)?;

        let spawned: HashSet<_> = self
            .storage
            .get(&project_id)
            .map(|v| v.iter().map(|s| s.name.clone()).collect())
            .unwrap_or_default();

        // TODO: Recalculate dependencies
        for service_name in services {
            let Some(service) = config.services.get(&service_name) else {
                return Err(Error::ServiceNotFound(project_id, service_name));
            };

            if spawned.contains(&service_name) {
                tracing::info!("Service {service_name:?} is already running");
                continue;
            }

            if service.deps.is_empty() {
                let proc_id = self
                    .start_service(service.clone(), service_name.clone(), project_id.clone())
                    .await?;

                self.storage
                    .entry(project_id.clone())
                    .or_default()
                    .push(RunningService {
                        name: service_name.clone(),
                        pid: Some(proc_id),
                        status: Status::Starting,
                    });
            } else {
                self.storage
                    .entry(project_id.clone())
                    .or_default()
                    .push(RunningService {
                        name: service_name.clone(),
                        pid: None,
                        status: Status::Waiting {
                            wait_for: service.deps.clone(),
                        },
                    });
            }
        }

        Ok(())
    }

    async fn down(&mut self, project_id: ProjectId) -> Result<()> {
        let services = self.storage.remove(&project_id).unwrap_or_default();
        for service in services {
            if service.status == Status::Running {
                if let Some(pid) = service.pid {
                    self.process_manager.kill(pid).await?;
                }
            }
        }
        Ok(())
    }

    async fn start_service(
        &mut self,
        service: Service,
        service_name: String,
        project_id: ProjectId,
    ) -> Result<ProcId> {
        tracing::debug!("Starting service {service_name:?} for project {project_id:?}");

        let process = self
            .process_manager
            .spawn(CommandSpec {
                name: service_name.clone(),
                cmd: service.cmd.clone(),
                cwd: service.cwd.clone(),
                env: service
                    .env
                    .clone()
                    .map(|h| h.into_iter().collect())
                    .unwrap_or_default(),
            })
            .await?;

        {
            let commands_tx = self.commands_tx.clone();
            let output_tx = self.output_tx.clone();
            let mut stdout = process.stdout;
            let project_id_clone = project_id.clone();
            let service_name_clone = service_name.clone();
            tokio::spawn(async move {
                while let Some(command) = stdout.next().await {
                    let log = String::from_utf8_lossy(&command);
                    if let Err(err) = output_tx
                        .send(SupervisorEvent::Log {
                            project_id: project_id_clone.clone(),
                            service: service_name_clone.clone(),
                            message: log.to_string(),
                        })
                        .await
                    {
                        tracing::error!("Failed to send log event: {}", err);
                    }
                }
                if let Err(err) = commands_tx
                    .send(SupervisorCommand::EndOfLogs {
                        project_id: project_id_clone.clone(),
                        service: service_name_clone.clone(),
                    })
                    .await
                {
                    tracing::error!("Failed to send end of logs command: {}", err);
                }
            });
        }

        {
            let commands_tx = self.commands_tx.clone();
            let output_tx = self.output_tx.clone();
            let mut stderr = process.stderr;
            let project_id_clone = project_id.clone();
            let service_name_clone = service_name.clone();
            tokio::spawn(async move {
                while let Some(command) = stderr.next().await {
                    let log = String::from_utf8_lossy(&command);
                    if let Err(err) = output_tx
                        .send(SupervisorEvent::Log {
                            project_id: project_id_clone.clone(),
                            service: service_name_clone.clone(),
                            message: log.to_string(),
                        })
                        .await
                    {
                        tracing::error!("Failed to send log event: {}", err);
                    }
                }
                if let Err(err) = commands_tx
                    .send(SupervisorCommand::EndOfLogs {
                        project_id: project_id_clone.clone(),
                        service: service_name_clone.clone(),
                    })
                    .await
                {
                    tracing::error!("Failed to send end of logs command: {}", err);
                }
            });
        }

        {
            let commands_tx = self.commands_tx.clone();
            let healthcheck = service.healthcheck;
            let project_id_clone = project_id.clone();
            let service_name_clone = service_name.clone();
            tokio::spawn(async move {
                if healthcheck.is_none() {
                    let _ = commands_tx
                        .send(SupervisorCommand::HealthCheckSuccess {
                            project_id: project_id_clone.clone(),
                            service: service_name_clone.clone(),
                        })
                        .await;
                }
            });
        }

        Ok(process.id)
    }

    fn end_of_logs(&mut self, project_id: ProjectId, service: String) -> Result<()> {
        let Some(running_services) = self.storage.get_mut(&project_id) else {
            return Err(Error::ProjectNotFound(project_id));
        };
        let Some(service) = running_services.iter_mut().find(|s| s.name == service) else {
            return Err(Error::ServiceNotFound(project_id, service));
        };

        service.status = Status::Stopped;

        // TODO: Restart policy

        Ok(())
    }

    async fn health_check_success(
        &mut self,
        project_id: ProjectId,
        updated_service: String,
    ) -> Result<()> {
        // TODO: too many clones
        let Some(mut running_services) = self.storage.get(&project_id).cloned() else {
            return Err(Error::ProjectNotFound(project_id));
        };

        {
            let Some(service) = running_services
                .iter_mut()
                .find(|s| s.name == updated_service)
            else {
                return Err(Error::ServiceNotFound(project_id, updated_service));
            };

            service.status = Status::Running;
        }

        for running_service in &mut running_services {
            if let Status::Waiting { wait_for } = &mut running_service.status {
                let new_wait_for = wait_for
                    .clone()
                    .into_iter()
                    .filter(|item| item != &updated_service)
                    .collect();

                *wait_for = new_wait_for;

                if wait_for.is_empty() {
                    running_service.status = Status::Running;

                    let Some(config) = self.config.get(&project_id).cloned() else {
                        return Err(Error::ProjectNotFound(project_id));
                    };
                    let Some(service) = config.services.get(&running_service.name) else {
                        return Err(Error::ServiceNotFound(
                            project_id,
                            running_service.name.clone(),
                        ));
                    };

                    let proc_id = self
                        .start_service(
                            service.clone(),
                            running_service.name.clone(),
                            project_id.clone(),
                        )
                        .await?;

                    running_service.pid = Some(proc_id);
                    running_service.status = Status::Starting;
                }
            }
        }

        self.storage.insert(project_id, running_services);

        Ok(())
    }

    fn toposort(config: &Project, services: &[String]) -> Result<Vec<String>> {
        let project_id = config.id.clone();

        let mut to_process = VecDeque::with_capacity(config.services.len());
        let mut processed = HashSet::with_capacity(config.services.len());
        let mut graph: HashMap<String, Vec<String>> = HashMap::with_capacity(config.services.len());
        let mut deps_count: HashMap<String, usize> = HashMap::with_capacity(config.services.len());

        for s in services {
            to_process.push_back(s.clone());
        }

        while let Some(service_name) = to_process.pop_front() {
            if !processed.insert(service_name.clone()) {
                continue;
            }
            let Some(service) = config.services.get(&service_name) else {
                return Err(Error::ServiceNotFound(project_id, service_name));
            };

            graph.entry(service_name.clone()).or_default();
            let deps_count = deps_count.entry(service_name.clone()).or_default();

            for dependency in &service.deps {
                if !config.services.contains_key(dependency) {
                    return Err(Error::ServiceNotFound(project_id, dependency.clone()));
                }
                graph
                    .entry(dependency.clone())
                    .or_default()
                    .push(service_name.clone());
                *deps_count += 1;
                to_process.push_back(dependency.clone());
            }
        }

        let mut zeros: Vec<String> = deps_count
            .iter()
            .filter_map(|(s, &c)| if c == 0 { Some(s.clone()) } else { None })
            .collect();
        zeros.sort();

        let mut queue: VecDeque<String> = zeros.into();

        for deps in graph.values_mut() {
            deps.sort();
        }

        let mut result = Vec::with_capacity(deps_count.len());
        while let Some(service) = queue.pop_front() {
            result.push(service.clone());

            if let Some(dependents) = graph.get(&service) {
                for dep in dependents {
                    if let Some(c) = deps_count.get_mut(dep) {
                        *c -= 1;
                        if *c == 0 {
                            let mut v: Vec<_> = queue.into();
                            v.push(dep.clone());
                            v.sort();
                            queue = v.into();
                        }
                    }
                }
            }
        }

        if result.len() != processed.len() {
            return Err(Error::CircularDependencyDetected);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use tutti_types::{ProjectId, Service};

    use crate::process_manager::MockProcessManager;

    use super::*;

    #[tokio::test]
    async fn test_toposort() {
        let project_id = ProjectId("/project".parse().unwrap());

        let result = SupervisorBackground::<MockProcessManager>::toposort(
            &Project {
                version: 1,
                id: project_id.clone(),
                services: vec![
                    (
                        "A".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec!["B".to_string(), "C".to_string()],
                            healthcheck: None,
                        },
                    ),
                    (
                        "B".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec![],
                            healthcheck: None,
                        },
                    ),
                    (
                        "C".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec!["D".to_string(), "E".to_string()],
                            healthcheck: None,
                        },
                    ),
                    (
                        "D".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec!["F".to_string()],
                            healthcheck: None,
                        },
                    ),
                    (
                        "E".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec![],
                            healthcheck: None,
                        },
                    ),
                    (
                        "F".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec![],
                            healthcheck: None,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
            },
            &vec!["A".to_string()],
        )
        .unwrap();

        assert_eq!(result, vec!["B", "E", "F", "D", "C", "A"]);
    }
}
