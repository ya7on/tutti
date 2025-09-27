use std::time::Duration;

use futures::StreamExt;
use libc::{killpg, setsid, SIGKILL, SIGTERM};
use tokio::{
    io::BufReader,
    process::{Child, Command},
    time::{sleep, Instant},
};
use tokio_util::io::ReaderStream;

use super::{CommandSpec, ProcId, ProcessManager, Spawned};

struct ChildRec {
    child: Child,
    pgid: libc::pid_t,
}

pub struct UnixProcessManager {
    // TODO: refactor
    processes: Vec<Option<ChildRec>>,
}

#[async_trait::async_trait]
impl ProcessManager for UnixProcessManager {
    async fn spawn(&mut self, spec: CommandSpec) -> anyhow::Result<Spawned> {
        anyhow::ensure!(
            !spec.cmd.is_empty(),
            "empty cmd for service `{}`",
            spec.name
        );

        let mut cmd = Command::new(&spec.cmd[0]);
        if spec.cmd.len() > 1 {
            cmd.args(&spec.cmd[1..]);
        }
        if let Some(dir) = &spec.cwd {
            cmd.current_dir(dir);
        }
        for (k, v) in &spec.env {
            cmd.env(k, v);
        }

        #[allow(unsafe_code)]
        unsafe {
            cmd.pre_exec(|| {
                if setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;

        let pid = child.id();
        let pgid = pid.ok_or_else(|| anyhow::anyhow!("spawned process has no pid"))? as libc::pid_t;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("stdout not piped"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("stderr not piped"))?;

        let out_stream = ReaderStream::new(BufReader::new(stdout))
            .filter_map(|res| async move { res.ok().map(|b| b.to_vec()) });
        let err_stream = ReaderStream::new(BufReader::new(stderr))
            .filter_map(|res| async move { res.ok().map(|b| b.to_vec()) });

        let id = ProcId(self.processes.len() as u64);
        self.processes.push(Some(ChildRec { child, pgid }));

        Ok(Spawned {
            id,
            pid,
            stdout: Box::pin(out_stream),
            stderr: Box::pin(err_stream),
        })
    }

    async fn shutdown(&mut self, id: ProcId) -> anyhow::Result<()> {
        let proc = self
            .processes
            .get(id.0 as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown process id {:?}", id))?
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("already shutdown process id {:?}", id))?;

        #[allow(unsafe_code)]
        unsafe {
            let rc = killpg(proc.pgid, SIGTERM);
            if rc == -1 {
                return Err(std::io::Error::last_os_error().into());
            }
        }

        Ok(())
    }

    async fn wait(&mut self, id: ProcId, d: Duration) -> anyhow::Result<Option<i32>> {
        let proc = self
            .processes
            .get_mut(id.0 as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown process id {:?}", id))?
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("already shutdown process id {:?}", id))?;
        let start = Instant::now();
        loop {
            if let Ok(Some(code)) = proc.child.try_wait() {
                self.processes[id.0 as usize] = None;
                return Ok(Some(code.code().unwrap_or_default()));
            }

            if start.elapsed() >= d {
                return Ok(None);
            }
            sleep(Duration::from_millis(50)).await;
        }
    }

    async fn kill(&mut self, id: ProcId) -> anyhow::Result<()> {
        let proc = self
            .processes
            .get(id.0 as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown process id {:?}", id))?
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("already shutdown process id {:?}", id))?;

        #[allow(unsafe_code)]
        unsafe {
            let rc = killpg(proc.pgid, SIGKILL);
            if rc == -1 {
                return Err(std::io::Error::last_os_error().into());
            }
        }

        let _ = self.wait(id, Duration::from_millis(10)).await;
        Ok(())
    }
}
