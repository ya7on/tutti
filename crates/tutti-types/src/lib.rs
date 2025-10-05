use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ProjectId(pub PathBuf);

#[derive(Debug)]
pub struct Project {
    pub version: u32,
    pub id: ProjectId,
    pub services: BTreeMap<String, Service>,
}

#[derive(Debug)]
pub struct Service {
    pub cmd: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub deps: Vec<String>,
}
