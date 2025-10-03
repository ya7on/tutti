use std::time::Duration;

use crate::process_manager::types::{CommandSpec, ProcId, Spawned};

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
