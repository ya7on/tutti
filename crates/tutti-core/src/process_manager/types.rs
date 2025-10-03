use std::{fmt::Debug, path::PathBuf, pin::Pin};

use futures::Stream;

pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

#[derive(Clone, Debug)]
pub struct CommandSpec {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Vec<(String, String)>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProcId(pub u64);

pub struct Spawned {
    pub id: ProcId,
    pub pid: Option<u32>,
    pub stdout: BoxStream<Vec<u8>>,
    pub stderr: BoxStream<Vec<u8>>,
}

impl Debug for Spawned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spawned")
            .field("id", &self.id)
            .field("pid", &self.pid)
            .field("stdout", &"<stream>")
            .field("stderr", &"<stream>")
            .finish()
    }
}
