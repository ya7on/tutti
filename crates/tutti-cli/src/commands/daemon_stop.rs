use std::path::PathBuf;

use anyhow::Result;
use tutti_daemon::DaemonRunner;
use tutti_transport::client::ipc_client::IpcClient;

use crate::DEFAULT_SYSTEM_DIR;

pub async fn daemon_stop(system_directory: Option<String>) -> Result<()> {
    let system_directory =
        system_directory.map_or_else(|| PathBuf::from(DEFAULT_SYSTEM_DIR), PathBuf::from);

    let daemon_runner = DaemonRunner::new(system_directory);

    let mut client = match IpcClient::new(daemon_runner.socket_path()).await {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to connect to the daemon: {err:?}");
            return Ok(());
        }
    };

    if client.shutdown().await.is_err() {
        println!("Failed to start project");
    }

    Ok(())
}
