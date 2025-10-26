use std::{collections::BTreeMap, path::Path};

use tutti_types::{Project, ProjectId, Restart, Service};

use crate::{
    raw::{RawProject, RawRestart},
    ConfigError,
};

impl RawProject {
    pub fn to_project(&self, path: &Path) -> Result<Project, ConfigError> {
        let services = self
            .services
            .iter()
            .map(|(name, raw_service)| {
                if raw_service.cmd.is_empty() {
                    return Err(ConfigError::Validation(format!(
                        "service `{name}`: cmd is empty"
                    )));
                }
                if raw_service.cmd.iter().any(|c| c.trim().is_empty()) {
                    return Err(ConfigError::Validation(format!(
                        "service `{name}`: cmd contains empty element"
                    )));
                }
                // TODO: Add validations

                let restart = raw_service
                    .restart
                    .as_ref()
                    .map(|policy| match policy {
                        RawRestart::Always => Restart::Always,
                        RawRestart::Never => Restart::Never,
                    })
                    .unwrap_or_default();

                Ok((
                    name.clone(),
                    Service {
                        cmd: raw_service.cmd.clone(),
                        cwd: raw_service.cwd.clone().and_then(|cwd| cwd.parse().ok()),
                        env: raw_service.env.clone(),
                        deps: raw_service.deps.clone().unwrap_or_default(),
                        healthcheck: raw_service.healthcheck,
                        restart,
                    },
                ))
            })
            .collect::<Result<BTreeMap<String, Service>, ConfigError>>()?;

        Ok(Project {
            id: ProjectId(path.into()),
            version: self.version,
            services,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::raw::RawService;

    use super::*;

    #[test]
    fn test_raw_to_project_success() {
        let raw = {
            let mut services = BTreeMap::new();
            services.insert(
                "full_service".into(),
                RawService {
                    cmd: vec!["echo".to_owned(), "hello".to_owned()],
                    cwd: Some("/tmp".to_owned()),
                    env: Some(HashMap::from_iter(vec![(
                        "KEY".to_owned(),
                        "Value".to_owned(),
                    )])),
                    deps: Some(vec!["empty_service".to_owned()]),
                    healthcheck: None,
                    restart: Some(RawRestart::Always),
                },
            );
            services.insert(
                "empty_service".into(),
                RawService {
                    cmd: vec!["echo".to_owned(), "hello".to_owned()],
                    cwd: None,
                    env: None,
                    deps: None,
                    healthcheck: None,
                    restart: None,
                },
            );
            RawProject {
                version: 1,
                services: services,
            }
        };
        let expected = {
            let mut services = BTreeMap::new();
            services.insert(
                "full_service".into(),
                Service {
                    cmd: vec!["echo".to_owned(), "hello".to_owned()],
                    cwd: Some(PathBuf::from("/tmp")),
                    env: Some(HashMap::from_iter(vec![(
                        "KEY".to_owned(),
                        "Value".to_owned(),
                    )])),
                    deps: vec!["empty_service".to_owned()],
                    healthcheck: None,
                    restart: Restart::Always,
                },
            );
            services.insert(
                "empty_service".into(),
                Service {
                    cmd: vec!["echo".to_owned(), "hello".to_owned()],
                    cwd: None,
                    env: None,
                    deps: vec![],
                    healthcheck: None,
                    restart: Restart::Never,
                },
            );
            Project {
                id: ProjectId("test".into()),
                version: 1,
                services: services,
            }
        };

        let actual = raw.to_project(&PathBuf::from("test")).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_empty_cmd() {
        {
            let raw = {
                let mut services = BTreeMap::new();
                services.insert(
                    "test".into(),
                    RawService {
                        cmd: vec![],
                        cwd: None,
                        env: None,
                        deps: None,
                        healthcheck: None,
                        restart: None,
                    },
                );
                RawProject {
                    version: 1,
                    services: services,
                }
            };
            let result = raw.to_project(&PathBuf::from("test"));
            assert!(result.is_err());
        }
        {
            let raw = {
                let mut services = BTreeMap::new();
                services.insert(
                    "test".into(),
                    RawService {
                        cmd: vec!["echo".to_owned(), "".to_owned()],
                        cwd: None,
                        env: None,
                        deps: None,
                        healthcheck: None,
                        restart: None,
                    },
                );
                RawProject {
                    version: 1,
                    services: services,
                }
            };
            let result = raw.to_project(&PathBuf::from("test"));
            assert!(result.is_err());
        }
    }
}
