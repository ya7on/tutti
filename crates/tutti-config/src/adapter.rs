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
