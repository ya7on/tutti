use std::path::PathBuf;

use anyhow::Result;
use tokio::signal;
use tutti_config::load_from_path;
use tutti_daemon::DaemonRunner;
use tutti_transport::{api::TuttiApi, client::ipc_client::IpcClient};

use crate::{logger::Logger, DEFAULT_FILENAMES, DEFAULT_SYSTEM_DIR};

pub async fn run(
    file: Option<String>,
    mut services: Vec<String>,
    system_directory: Option<String>,
    _kill_timeout: Option<u64>,
) -> Result<()> {
    let file = file.unwrap_or_else(|| {
        for filename in DEFAULT_FILENAMES {
            if std::path::Path::new(filename).exists() {
                return filename.to_string();
            }
        }
        "tutti.toml".to_string()
    });

    let system_directory =
        system_directory.map_or_else(|| PathBuf::from(DEFAULT_SYSTEM_DIR), PathBuf::from);

    let daemon_runner = DaemonRunner::new(system_directory);
    if daemon_runner.prepare().is_err() {
        println!("Failed to prepare daemon");
        return Ok(());
    }

    if IpcClient::check_socket(&daemon_runner.socket_path()).await {
        tracing::debug!("Daemon already running");
    } else {
        tracing::debug!("Starting daemon");
        if let Err(err) = daemon_runner.spawn() {
            println!("Failed to spawn daemon: {err:?}");
        }
    }

    let path = PathBuf::from(file);
    let project = load_from_path(&path)?;
    let project_id = project.id.clone();

    let mut client = match IpcClient::new(daemon_runner.socket_path()).await {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to connect to the daemon: {err:?}");
            return Ok(());
        }
    };

    if services.is_empty() {
        services = project
            .services
            .keys()
            .map(std::borrow::ToOwned::to_owned)
            .collect();
    }

    if client.up(project, services).await.is_err() {
        println!("Failed to start project");
    }

    let Ok(mut logs) = client.subscribe().await else {
        println!("Failed to subscribe to logs");
        return Ok(());
    };

    let mut shutting_down = false;
    let mut logger = Logger::default();

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                if shutting_down {
                    tracing::warn!("Second Ctrl+C: exiting immediately");
                    return Ok(());
                }

                shutting_down = true;
                tracing::info!("Ctrl+C: stopping services (sending Down)...");

                if let Err(err) = client.down(project_id.clone()).await {
                    tracing::error!("Failed to send Down: {err:?}");
                    return Ok(());
                }
            }

            maybe_msg = logs.recv() => {
                if let Some(message) = maybe_msg {
                    match message.body {
                        TuttiApi::ProjectStopped { project_id } => {
                            tracing::info!("Project stopped: {}", project_id);
                            logger.system("All services stopped");
                            return Ok(());
                        }
                        TuttiApi::ServiceStopped { project_id: _, service } => {
                            logger.system(&format!("Service stopped: {service}"));
                        }
                        TuttiApi::ServiceRestarted { project_id: _, service } => {
                            logger.system(&format!("Service restarted: {service}"));
                        }
                        TuttiApi::Log { project_id: _, service, message } => {
                            logger.log(&service, &message);
                        }
                        TuttiApi::Error { project_id: _, message } => {
                            logger.error(&message);
                            return Ok(());
                        }
                        _ => {}
                    }
                } else {
                    tracing::info!("Log stream ended");
                    return Ok(())
                }
            }
        }
    }
}
