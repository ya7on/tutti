use std::time::Duration;

use futures::StreamExt;
use libc::{killpg, setsid, SIGINT, SIGKILL};
use tokio::{
    io::BufReader,
    process::{Child, Command},
    time::{sleep, Instant},
};
use tokio_util::io::ReaderStream;

use crate::{
    error::{Error, Result},
    process_manager::{
        base::ProcessManager,
        types::{CommandSpec, ProcId, Spawned},
    },
};

#[derive(Debug)]
struct ChildRec {
    child: Child,
    pgid: libc::pid_t,
}

/// Unix-specific process manager.
#[derive(Debug)]
pub struct UnixProcessManager {
    // TODO: refactor
    processes: Vec<Option<ChildRec>>,
}

impl Default for UnixProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UnixProcessManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl ProcessManager for UnixProcessManager {
    async fn spawn(&mut self, spec: CommandSpec) -> Result<Spawned> {
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

        let mut child = cmd.spawn().map_err(Error::IOError)?;

        let pid = child.id();

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::IOError(std::io::Error::other("stdout not piped")))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| Error::IOError(std::io::Error::other("stdout not piped")))?;

        let out_stream = ReaderStream::new(BufReader::new(stdout))
            .filter_map(|res| async move { res.ok().map(|b| b.to_vec()) });
        let err_stream = ReaderStream::new(BufReader::new(stderr))
            .filter_map(|res| async move { res.ok().map(|b| b.to_vec()) });

        let id = ProcId(self.processes.len() as u64);
        self.processes.push(Some(ChildRec {
            child,
            pgid: libc::pid_t::try_from(
                pid.ok_or_else(|| Error::IOError(std::io::Error::other("pid not available")))?,
            )
            .map_err(|_| Error::IOError(std::io::Error::other("pid not available")))?,
        }));

        Ok(Spawned {
            id,
            pid,
            stdout: Box::pin(out_stream),
            stderr: Box::pin(err_stream),
        })
    }

    async fn shutdown(&mut self, id: ProcId) -> Result<()> {
        let proc = self
            .processes
            .get(usize::try_from(id.0).map_err(|_| {
                Error::IOError(std::io::Error::other("Cannot convert process id to usize"))
            })?)
            .ok_or_else(|| Error::IOError(std::io::Error::other("unknown process id {id:?}")))?
            .as_ref()
            .ok_or_else(|| {
                Error::IOError(std::io::Error::other("already shutdown process id {id:?}"))
            })?;

        #[allow(unsafe_code)]
        unsafe {
            let rc = killpg(proc.pgid, SIGINT);
            if rc == -1 {
                return Err(Error::IOError(std::io::Error::last_os_error()));
            }
        }

        Ok(())
    }

    async fn wait(&mut self, id: ProcId, d: Duration) -> Result<Option<i32>> {
        let index = usize::try_from(id.0).map_err(|_| {
            Error::IOError(std::io::Error::other("Cannot convert process id to usize"))
        })?;
        let proc = self
            .processes
            .get_mut(index)
            .ok_or_else(|| Error::IOError(std::io::Error::other("unknown process id {id:?}")))?
            .as_mut()
            .ok_or_else(|| {
                Error::IOError(std::io::Error::other("already shutdown process id {id:?}"))
            })?;

        let start = Instant::now();
        loop {
            if let Ok(Some(code)) = proc.child.try_wait() {
                self.processes[index] = None;
                return Ok(Some(code.code().unwrap_or_default()));
            }

            if start.elapsed() >= d {
                return Ok(None);
            }
            sleep(Duration::from_millis(50)).await;
        }
    }

    async fn kill(&mut self, id: ProcId) -> Result<()> {
        let proc = self
            .processes
            .get(usize::try_from(id.0).map_err(|_| {
                Error::IOError(std::io::Error::other("Cannot convert process id to usize"))
            })?)
            .ok_or_else(|| Error::IOError(std::io::Error::other("unknown process id {id:?}")))?
            .as_ref()
            .ok_or_else(|| {
                Error::IOError(std::io::Error::other("already shutdown process id {id:?}"))
            })?;

        #[allow(unsafe_code)]
        unsafe {
            let rc = killpg(proc.pgid, SIGKILL);
            if rc == -1 {
                return Err(Error::IOError(std::io::Error::last_os_error()));
            }
        }

        Ok(())
    }
}
