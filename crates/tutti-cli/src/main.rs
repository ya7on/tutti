use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::Result;
use clap::Parser;
use colored::{Color, Colorize};
use tokio::sync::mpsc;
use tutti_config::load_from_path;
use tutti_core::{Supervisor, SupervisorEvent, UnixProcessManager};

mod config;

const DEFAULT_FILENAMES: [&str; 3] = ["tutti.toml", "tutti.config.toml", "Tutti.toml"];

fn string_to_color(s: &str) -> Color {
    let colors = [
        Color::Green,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::BrightGreen,
        Color::BrightBlue,
        Color::BrightMagenta,
        Color::BrightCyan,
    ];

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();

    let idx = usize::try_from(hash).unwrap_or_default() % colors.len();
    colors[idx]
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = config::Cli::parse();

    match cli.command {
        config::Commands::Run {
            file,
            services,
            kill_timeout,
        } => {
            let file = file.unwrap_or_else(|| {
                for filename in DEFAULT_FILENAMES {
                    if std::path::Path::new(filename).exists() {
                        return filename.to_string();
                    }
                }
                "tutti.toml".to_string()
            });
            let path = std::path::Path::new(&file);
            let project = load_from_path(path)?;

            if !services.is_empty() {
                for name in &services {
                    if !project.services.contains_key(name) {
                        return Err(anyhow::anyhow!("Service {name} not found"));
                    }
                }
            }

            let process_manager = UnixProcessManager::new();
            let (mut supervisor, mut logs) = Supervisor::new(process_manager).await;

            supervisor.up(project.clone(), services).await.unwrap();

            let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1); // TODO watch

            tokio::spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() {
                    let line = "Received Ctrl+C, shutting down services...".yellow();
                    println!("\n{line}");
                    let _ = shutdown_tx.send(()).await;
                }
            });

            tokio::spawn(async move {
                while let Some(log) = logs.recv().await {
                    match log {
                        SupervisorEvent::Log {
                            message, service, ..
                        } => {
                            for line in message.lines() {
                                let prefix =
                                    format!("[{service}]").color(string_to_color(&service));
                                println!("{prefix} {line}");
                            }
                        }
                    }
                }
            });

            let _ = shutdown_rx.recv().await;
            supervisor.down(project).await;
        }
    }

    Ok(())
}
