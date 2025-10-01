use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::Result;
use clap::Parser;
use colored::{Color, Colorize};
use tokio::sync::mpsc;
use tutti_config::load_from_path;
use tutti_core::{LogEvent, Runner, UnixProcessManager};

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
        config::Commands::Run { file, services } => {
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
            let mut runner = Runner::new(project, process_manager);

            let mut logs = runner.up(services).await?;

            let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

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
                        LogEvent::Log { service_name, line } => {
                            let string = String::from_utf8_lossy(&line);
                            for line in string.lines() {
                                let prefix = format!("[{service_name}]")
                                    .color(string_to_color(&service_name));
                                println!("{prefix} {line}");
                            }
                        }
                        LogEvent::Stop { service_name } => {
                            let line = format!("{service_name} stopped")
                                .color(string_to_color(&service_name));
                            println!("{line}");
                        }
                    }
                }
            });

            tokio::select! {
                result = runner.wait() => {
                    result?;
                }
                _ = shutdown_rx.recv() => {
                    if let Err(err) = runner.down().await {
                        eprintln!("Error during shutdown: {err}");
                    }
                }
            }
        }
    }

    Ok(())
}
