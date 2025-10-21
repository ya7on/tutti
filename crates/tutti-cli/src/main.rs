use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tutti_config::load_from_path;
use tutti_daemon::{DaemonRunner, DEFAULT_SYSTEM_DIR, SOCKET_FILE};
use tutti_transport::{api::TuttiApi, client::ipc_client::IpcClient};

mod config;
mod logger;

const DEFAULT_FILENAMES: [&str; 3] = ["tutti.toml", "tutti.config.toml", "Tutti.toml"];

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
            let socket_file = system_directory.join(SOCKET_FILE);

            // let daemon_runner = DaemonRunner::new(system_directory.as_ref().map(PathBuf::from));
            // if daemon_runner.prepare().is_err() {
            //     println!("Failed to prepare daemon");
            //     return Ok(());
            // }

            let path = PathBuf::from(file);
            // if !IpcClient::check_socket(&path).await && daemon_runner.spawn().is_err() {
            //     println!("Failed to spawn daemon");
            // }

            let project = load_from_path(&path)?;

            let mut client = match IpcClient::new(socket_file).await {
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

            while let Some(message) = logs.recv().await {
                if let TuttiApi::Log {
                    project_id: _,
                    service,
                    message,
                } = message.body
                {
                    logger::Logger::log(&service, &message);
                }
            }
        }
        config::Commands::Daemon { system_directory } => {
            let daemon_runner = DaemonRunner::new(system_directory.as_ref().map(PathBuf::from));
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
