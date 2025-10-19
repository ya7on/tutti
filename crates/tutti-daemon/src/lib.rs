use std::{path::PathBuf, sync::Arc};

use futures_util::FutureExt;
use tokio::sync::{mpsc::Receiver, Mutex};
use tutti_core::{Supervisor, SupervisorEvent, UnixProcessManager};
use tutti_transport::{
    api::TuttiApi,
    error::{TransportError, TransportResult},
    server::ipc_server::IpcServer,
};

pub const SOCKET_FILE: &str = "tutti.sock";

pub const DEFAULT_SYSTEM_DIR: &str = "~/.tutti/";

#[derive(Debug, Clone)]
struct Context {
    supervisor: Arc<Mutex<Supervisor>>,
    receiver: Arc<Mutex<Receiver<SupervisorEvent>>>,
}

impl Context {
    pub fn new(
        supervisor: Arc<Mutex<Supervisor>>,
        receiver: Arc<Mutex<Receiver<SupervisorEvent>>>,
    ) -> Self {
        Context {
            supervisor,
            receiver,
        }
    }
}

async fn unary_handler(message: TuttiApi, context: Context) -> TransportResult<TuttiApi> {
    match message {
        TuttiApi::Ping => Ok(TuttiApi::Pong),
        TuttiApi::Up { project, services } => {
            tracing::info!("Starting project {project:?} with services {services:?}");

            let mut guard = context.supervisor.lock().await;
            guard
                .up(project, services)
                .await
                .map_err(|_| TransportError::UnknownMessage)?;

            Ok(TuttiApi::Pong)
        }
        _ => Err(TransportError::UnknownMessage),
    }
}

async fn stream_handler(context: Context) -> TransportResult<TuttiApi> {
    tracing::info!("Starting stream handler");

    let mut guard = context.receiver.lock().await;
    let Some(event) = guard.recv().await else {
        return Err(TransportError::UnknownMessage);
    };

    tracing::info!("Received event: {:?}", event);

    match event {
        SupervisorEvent::Log {
            project_id,
            service,
            message,
        } => Ok(TuttiApi::Log {
            project_id,
            service,
            message,
        }),
    }
}

#[derive(Debug)]
pub struct DaemonRunner {
    system: PathBuf,
}

impl DaemonRunner {
    #[must_use]
    pub fn new(system_dir: Option<PathBuf>) -> Self {
        DaemonRunner {
            system: system_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_SYSTEM_DIR)),
        }
    }

    /// Prepare the system directory.
    ///
    /// # Errors
    /// Returns an error if the system directory cannot be prepared.
    pub fn prepare(&self) -> Result<(), String> {
        if !std::fs::exists(&self.system)
            .map_err(|err| format!("Cannot prepare system directory: {err:?}"))?
        {
            std::fs::create_dir_all(&self.system)
                .map_err(|err| format!("Cannot create system directory: {err:?}"))?;
        }

        Ok(())
    }

    /// Spawn the daemon process.
    ///
    /// # Errors
    /// Returns an error if the daemon process cannot be spawned.
    pub fn spawn(&self) -> Result<(), String> {
        std::process::Command::new("tutti-cli")
            .arg("daemon")
            .arg("--run")
            .spawn()
            .map_err(|err| format!("Cannot spawn daemon process: {err:?}"))?;

        Ok(())
    }

    /// Start the daemon process.
    ///
    /// # Errors
    /// Returns an error if the daemon process cannot be started.
    #[tracing::instrument(skip_all)]
    pub async fn start(&self) -> Result<(), String> {
        tracing::info!("Starting daemon process...");
        let (supervisor, receiver) = Supervisor::new(UnixProcessManager::new());
        tracing::debug!("Supervisor created");

        let unary_handler =
            Arc::new(|api: TuttiApi, context: Context| unary_handler(api, context).boxed());
        let stream_handler = Arc::new(|context: Context| stream_handler(context).boxed());

        IpcServer::<Context>::new(
            self.system.join(SOCKET_FILE),
            Context::new(
                Arc::new(Mutex::new(supervisor)),
                Arc::new(Mutex::new(receiver)),
            ),
        )
        .map_err(|err| format!("Cannot start IPC Server: {err:?}"))?
        .add_unary_handler(unary_handler)
        .add_stream_handler(stream_handler)
        .start()
        .await;

        Ok(())
    }
}
