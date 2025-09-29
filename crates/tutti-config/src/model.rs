use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use crate::{raw::RawProject, ConfigError};

#[derive(Debug)]
pub struct Project {
    pub version: u32,
    pub services: BTreeMap<String, Service>,
}

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub deps: Vec<String>,
}

impl TryFrom<RawProject> for Project {
    type Error = ConfigError;

    fn try_from(raw_project: RawProject) -> Result<Self, Self::Error> {
        let services = raw_project
            .services
            .into_iter()
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

                Ok((
                    name.clone(),
                    Service {
                        name,
                        cmd: raw_service.cmd,
                        cwd: raw_service.cwd.and_then(|cwd| cwd.parse().ok()),
                        env: raw_service.env,
                        deps: raw_service.deps.unwrap_or_default(),
                    },
                ))
            })
            .collect::<Result<BTreeMap<String, Service>, Self::Error>>()?;

        Ok(Project {
            version: raw_project.version,
            services,
        })
    }
}
