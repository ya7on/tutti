use std::collections::BTreeMap;

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
