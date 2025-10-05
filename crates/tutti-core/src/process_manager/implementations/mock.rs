use std::time::Duration;

use async_trait::async_trait;
use tokio_stream::wrappers::ReceiverStream;

use crate::{CommandSpec, ProcId, ProcessManager, Spawned};

#[derive(Default)]
pub struct MockProcessManager {
    storage: Vec<CommandSpec>,
}

#[async_trait]
impl ProcessManager for MockProcessManager {
    async fn spawn(&mut self, spec: CommandSpec) -> anyhow::Result<Spawned> {
        self.storage.push(spec);
        let (_, stdout) = tokio::sync::mpsc::channel(1);
        let (_, stderr) = tokio::sync::mpsc::channel(1);
        Ok(Spawned {
            id: ProcId(0),
            stdout: Box::pin(ReceiverStream::new(stdout)),
            stderr: Box::pin(ReceiverStream::new(stderr)),
            pid: None,
        })
    }
    async fn shutdown(&mut self, id: ProcId) -> anyhow::Result<()> {
        todo!()
    }
    async fn wait(&mut self, id: ProcId, d: Duration) -> anyhow::Result<Option<i32>> {
        todo!()
    }
    async fn kill(&mut self, id: ProcId) -> anyhow::Result<()> {
        todo!()
    }
}
