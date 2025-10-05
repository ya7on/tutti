use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc;
use tutti_types::{Project, ProjectId};

use crate::{
    process_manager::ProcessManager,
    supervisor::commands::{SupervisorCommand, UpResponse},
    CommandSpec, Spawned,
};

#[derive(Debug)]
pub struct RunningService {
    pub spawned: Spawned,
    pub name: String,
}

#[derive(Debug)]
pub struct Supervisor<P: ProcessManager> {
    process_manager: P,
    storage: HashMap<ProjectId, Vec<RunningService>>,
}

impl<P: ProcessManager> Supervisor<P> {
    pub fn new(process_manager: P) -> Self {
        Self {
            process_manager,
            storage: HashMap::new(),
        }
    }

    pub async fn run(&mut self, mut rx: mpsc::Receiver<SupervisorCommand>) {
        while let Some(command) = rx.recv().await {
            self.handle_command(command).await;
        }
    }

    async fn handle_command(&mut self, command: SupervisorCommand) {
        match command {
            SupervisorCommand::Up {
                project,
                services,
                resp,
            } => {
                self.up(project, services, resp).await;
            }
        }
    }

    async fn up(&mut self, project: Project, services: Vec<String>, resp: UpResponse) {
        tracing::trace!(
            "Received up command for project {project:?} to start services {services:?}"
        );

        let stored_project = self.storage.entry(project.id).or_default();
        let mut new_services = Vec::new();

        let already_spawned = stored_project
            .iter()
            .map(|s| &s.name)
            .collect::<HashSet<_>>();

        for (name, service) in project.services {
            if already_spawned.contains(&name) {
                tracing::debug!("Service {name} is already running. Skipping");
                continue;
            }

            let spawned = self
                .process_manager
                .spawn(CommandSpec {
                    name: name.clone(),
                    cmd: service.cmd,
                    cwd: service.cwd,
                    env: service
                        .env
                        .map(|(h)| h.into_iter().collect())
                        .unwrap_or_default(),
                })
                .await
                .unwrap();
            tracing::info!("Service {spawned:?} started");
            new_services.push(RunningService { spawned, name });
        }

        stored_project.extend(new_services);
    }
}

#[cfg(test)]
mod tests {
    use tutti_types::{ProjectId, Service};

    use crate::process_manager::MockProcessManager;

    use super::*;

    #[tokio::test]
    async fn test_up() {
        let mut supervisor = Supervisor::new(MockProcessManager::default());

        let (tx, _) = mpsc::channel(1);

        let project_id = ProjectId("/project".parse().unwrap());

        supervisor
            .up(
                Project {
                    id: project_id.clone(),
                    services: vec![(
                        "service1".to_string(),
                        Service {
                            cmd: vec!["echo".to_string(), "hello".to_string()],
                            cwd: Some("/".parse().unwrap()),
                            env: None,
                            deps: vec![],
                        },
                    )]
                    .into_iter()
                    .collect(),
                    version: 1,
                },
                vec!["service1".to_string()],
                tx,
            )
            .await;

        assert!(supervisor.storage.contains_key(&project_id));
        assert_eq!(
            supervisor.storage.get(&project_id).unwrap()[0].name,
            "service1"
        );
    }

    #[tokio::test]
    async fn test_toposort() {
        let mut supervisor = Supervisor::new(MockProcessManager::default());

        let (tx, _) = mpsc::channel(1);

        let project_id = ProjectId("/project".parse().unwrap());

        supervisor
            .up(
                Project {
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
                vec![],
                tx,
            )
            .await;

        assert!(supervisor.storage.contains_key(&project_id));
        assert_eq!(supervisor.storage.get(&project_id).unwrap().len(), 6);
        assert_eq!(
            supervisor
                .storage
                .get(&project_id)
                .unwrap()
                .into_iter()
                .map(|p| p.name.clone())
                .collect::<Vec<String>>(),
            vec!["B", "F", "E", "D", "C", "A"]
        );
    }
}
