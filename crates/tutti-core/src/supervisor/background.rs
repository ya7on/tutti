use std::collections::{HashMap, HashSet, VecDeque};

use tutti_types::{Project, ProjectId};

use crate::{
    error::{Error, Result},
    supervisor::{
        main::{RunningService, Status},
        SupervisorCommand,
    },
    CommandSpec, ProcessManager,
};

#[derive(Debug)]
pub struct SupervisorBackground<P: ProcessManager> {
    process_manager: P,
    storage: HashMap<ProjectId, Vec<RunningService>>,
    config: HashMap<ProjectId, Project>,

    commands_tx: tokio::sync::mpsc::Sender<SupervisorCommand>,
    commands_rx: tokio::sync::mpsc::Receiver<SupervisorCommand>,
}

impl<P: ProcessManager> SupervisorBackground<P> {
    pub fn new(
        process_manager: P,
        commands_tx: tokio::sync::mpsc::Sender<SupervisorCommand>,
        commands_rx: tokio::sync::mpsc::Receiver<SupervisorCommand>,
    ) -> Self {
        Self {
            process_manager,
            storage: HashMap::new(),
            config: HashMap::new(),
            commands_tx,
            commands_rx,
        }
    }

    pub async fn run(&mut self) {
        while let Some(command) = self.commands_rx.recv().await {
            if let Err(err) = self.handle_commands(command).await {
                tracing::error!("Error handling command: {err:?}");
            }
        }
    }

    async fn handle_commands(&mut self, command: SupervisorCommand) -> Result<()> {
        match command {
            SupervisorCommand::UpdateConfig { project_id, config } => {
                tracing::info!("Updating config for project {project_id:?}");
                self.config.insert(project_id, config);

                Ok(())
            }
            SupervisorCommand::Up {
                project_id,
                services,
            } => {
                tracing::info!("Starting {services:?} services for project {project_id:?}");

                let Some(config) = self.config.get(&project_id) else {
                    return Err(Error::ProjectNotFound(project_id));
                };

                let services = Self::toposort(config, &services)?;
                let storage = self.storage.entry(project_id.clone()).or_default();
                let spawned = storage
                    .iter()
                    .map(|service| service.name.clone())
                    .collect::<HashSet<_>>();

                for service_name in services {
                    let Some(service) = config.services.get(&service_name) else {
                        return Err(Error::ServiceNotFound(project_id, service_name));
                    };

                    if spawned.contains(&service_name) {
                        tracing::info!("Service {service_name:?} is already running");
                        continue;
                    }

                    // TODO: Recalculate dependencies count for already running services
                    if service.deps.is_empty() {
                        let process = self
                            .process_manager
                            .spawn(CommandSpec {
                                name: service_name.to_owned(),
                                cmd: service.cmd.clone(),
                                cwd: service.cwd.clone(),
                                env: service
                                    .env
                                    .clone()
                                    .map(|h| h.into_iter().collect())
                                    .unwrap_or_default(),
                            })
                            .await?;

                        storage.push(RunningService {
                            name: service_name.clone(),
                            spawned: Some(process),
                            status: Status::Starting,
                        });
                    } else {
                        storage.push(RunningService {
                            name: service_name.clone(),
                            spawned: None,
                            status: Status::Waiting,
                        });
                    }
                }

                todo!()
            }
        }
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
                        },
                    ),
                    (
                        "B".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec![],
                        },
                    ),
                    (
                        "C".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec!["D".to_string(), "E".to_string()],
                        },
                    ),
                    (
                        "D".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec!["F".to_string()],
                        },
                    ),
                    (
                        "E".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec![],
                        },
                    ),
                    (
                        "F".to_string(),
                        Service {
                            cmd: vec!["echo".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec![],
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
