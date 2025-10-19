use std::{path::PathBuf, sync::Arc};

use futures_util::FutureExt;
use tokio::sync::Mutex;
use tutti_core::{Supervisor, UnixProcessManager};
use tutti_transport::{
    api::TuttiApi,
    error::{TransportError, TransportResult},
    server::ipc_server::IpcServer,
};

const LOCK_FILE: &str = "tutti.lock";
const PID_FILE: &str = "tutti.pid";
const SOCKET_FILE: &str = "tutti.sock";

const DEFAULT_SYSTEM_DIR: &str = "~/.tutti/";

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
        let proc = std::process::Command::new("tutti-cli")
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
        }

        impl Context {
            pub fn new(supervisor: Arc<Mutex<Supervisor>>) -> Self {
                Context { supervisor }
            }
        }

        async fn handler(message: TuttiApi, context: Context) -> TransportResult<TuttiApi> {
            match message {
                TuttiApi::Ping => Ok(TuttiApi::Pong),
                TuttiApi::Up => {
                    let guard = context.supervisor.lock().await;
                    // guard.up().await.unwrap();
                    Ok(TuttiApi::Up)
                }
                _ => Err(TransportError::UnknownMessage),
            }
        }

        let handler = Arc::new(|api: TuttiApi, context: Context| handler(api, context).boxed());
        let server = IpcServer::<Context>::new(
            self.system.join(SOCKET_FILE),
            Context::new(Arc::new(Mutex::new(supervisor))),
        )
        .add_handler(handler)
        .start()
        .await;

        Ok(())
    }
}
