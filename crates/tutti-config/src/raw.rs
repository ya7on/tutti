use std::collections::BTreeMap;

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
pub(crate) struct RawService {
    pub cmd: Vec<String>,
}
