use std::path::PathBuf;

use anyhow::Result;
use tutti_daemon::DaemonRunner;
use tutti_transport::client::ipc_client::IpcClient;

use crate::DEFAULT_SYSTEM_DIR;

pub async fn daemon_start(system_directory: Option<String>) -> Result<()> {
    let daemon_runner = DaemonRunner::new(
        system_directory.map_or_else(|| PathBuf::from(DEFAULT_SYSTEM_DIR), PathBuf::from),
    );

    if !IpcClient::check_socket(&daemon_runner.socket_path()).await
        && daemon_runner.clear().is_err()
    {
        println!("Failed to clear daemon");
        return Ok(());
    }

    if daemon_runner.prepare().is_err() {
        println!("Failed to prepare daemon");
        return Ok(());
    }

    if daemon_runner.start().await.is_err() {
        println!("Failed to start daemon");
        return Ok(());
    }

    Ok(())
}
