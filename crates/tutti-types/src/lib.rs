use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct ProjectId(pub PathBuf);

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub deps: Vec<String>,
}
