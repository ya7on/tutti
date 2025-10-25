use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct RawProject {
    #[serde(default = "default_version")]
    pub version: u32,
    pub services: BTreeMap<String, RawService>,
}

fn default_version() -> u32 {
    1
}

#[derive(Deserialize)]
pub(crate) enum RawRestart {
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "never")]
    Never,
}

#[derive(Deserialize)]
pub(crate) struct RawService {
    pub cmd: Vec<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub deps: Option<Vec<String>>,
    pub restart: Option<RawRestart>,
    pub healthcheck: Option<()>, // TODO
}
