use std::{path::PathBuf, sync::Arc};

use futures_util::FutureExt;
use tokio::sync::{mpsc::Receiver, Mutex};
use tutti_core::{Supervisor, SupervisorEvent, UnixProcessManager};
use tutti_transport::{
    api::TuttiApi,
    error::{TransportError, TransportResult},
    server::ipc_server::IpcServer,
};

const SOCKET_FILE: &str = "tutti.sock";

const DEFAULT_SYSTEM_DIR: &str = "~/.tutti/";

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

    pub fn prepare(&self) -> Result<(), String> {
        if !std::fs::exists(&self.system).unwrap() {
            std::fs::create_dir_all(&self.system).unwrap();
        }

        Ok(())
    }

    pub fn spawn(&self) -> Result<(), String> {
        std::process::Command::new("tutti-cli")
            .arg("daemon")
            .arg("--run")
            .spawn()
            .unwrap();

        Ok(())
    }

    pub async fn start(&self) -> Result<(), String> {
        let (supervisor, receiver) = Supervisor::new(UnixProcessManager::new()).await;

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
                    let mut guard = context.supervisor.lock().await;
                    guard.up(project, services).await.unwrap();
                    // Ok(TuttiApi::Up)
                    todo!()
                }
                _ => Err(TransportError::UnknownMessage),
            }
        }

        async fn stream_handler(context: Context) -> TransportResult<TuttiApi> {
            while let Some(event) = context.receiver.lock().await.recv().await {
                match event {
                    SupervisorEvent::Log {
                        project_id,
                        service,
                        message,
                    } => {
                        return Ok(TuttiApi::Log {
                            project_id,
                            service,
                            message,
                        });
                    }
                }
            }

            Err(TransportError::UnknownMessage)
        }

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
        .add_unary_handler(unary_handler)
        .add_stream_handler(stream_handler)
        .start()
        .await;

        Ok(())
    }
}
