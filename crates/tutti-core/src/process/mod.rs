use futures_core::Stream;
use std::fmt::Debug;
use std::pin::Pin;
use std::{path::PathBuf, time::Duration};

pub mod unix;

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

#[async_trait::async_trait]
pub trait ProcessManager: Send + Sync {
    /// Spawn a new process.
    async fn spawn(&mut self, spec: CommandSpec) -> anyhow::Result<Spawned>;
    /// Gracefully shutdown a process.
    async fn shutdown(&mut self, id: ProcId) -> anyhow::Result<()>;
    /// Wait for a process to exit.
    async fn wait(&mut self, id: ProcId, d: Duration) -> anyhow::Result<Option<i32>>;
    /// Forcefully kill a process.
    async fn kill(&mut self, id: ProcId) -> anyhow::Result<()>;
}
