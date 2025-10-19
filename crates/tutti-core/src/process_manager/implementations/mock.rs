use std::time::Duration;

use async_trait::async_trait;
use tokio_stream::wrappers::ReceiverStream;

use crate::{error::Result, CommandSpec, ProcId, ProcessManager, Spawned};

#[derive(Default)]
pub struct MockProcessManager {
    storage: Vec<CommandSpec>,
}

#[async_trait]
impl ProcessManager for MockProcessManager {
    async fn spawn(&mut self, spec: CommandSpec) -> Result<Spawned> {
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
    async fn shutdown(&mut self, _id: ProcId) -> Result<()> {
        todo!()
    }
    async fn wait(&mut self, _id: ProcId, _d: Duration) -> Result<Option<i32>> {
        todo!()
    }
    async fn kill(&mut self, _id: ProcId) -> Result<()> {
        todo!()
    }
}
