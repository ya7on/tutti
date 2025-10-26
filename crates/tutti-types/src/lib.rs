use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct ProjectId(pub PathBuf);

impl Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub version: u32,
    pub id: ProjectId,
    pub services: BTreeMap<String, Service>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum Restart {
    Always,
    #[default]
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    pub cmd: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub deps: Vec<String>,
    pub healthcheck: Option<()>, // TODO
    pub restart: Restart,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_id_display() {
        let project_id = ProjectId(PathBuf::from("/path/to/project"));
        assert_eq!(project_id.to_string(), "/path/to/project");
    }
}
