use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tokio::signal;
use tutti_config::load_from_path;
use tutti_daemon::DaemonRunner;
use tutti_transport::{api::TuttiApi, client::ipc_client::IpcClient};

mod config;
mod logger;

const DEFAULT_FILENAMES: [&str; 3] = ["tutti.toml", "tutti.config.toml", "Tutti.toml"];
const DEFAULT_SYSTEM_DIR: &str = "~/.tutti/";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = config::Cli::parse();

    tracing_subscriber::fmt::init();

    match cli.command {
        config::Commands::Run {
            file,
            services,
            system_directory,
            kill_timeout: _,
        } => {
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

            if client.up(project, services).await.is_err() {
                println!("Failed to start project");
            }

            let Ok(mut logs) = client.subscribe().await else {
                println!("Failed to subscribe to logs");
                return Ok(());
            };

            let mut shutting_down = false;

            loop {
                tokio::select! {
                    _ = signal::ctrl_c() => {
                        if shutting_down {
                            tracing::warn!("Second Ctrl+C: exiting immediately");
                            break;
                        }

                        shutting_down = true;
                        tracing::info!("Ctrl+C: stopping services (sending Down)...");

                        if let Err(err) = client.down(project_id.clone()).await {
                            tracing::error!("Failed to send Down: {err:?}");
                            break;
                        }
                    }

                    maybe_msg = logs.recv() => {
                        if let Some(message) = maybe_msg {
                            if let TuttiApi::Log { project_id: _, service, message } = message.body {
                                logger::Logger::log(&service, &message);
                            }
                        } else {
                            tracing::info!("Log stream ended");
                            break;
                        }
                    }
                }
            }
        }
        config::Commands::Daemon { system_directory } => {
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
        }
    }

    Ok(())
}
