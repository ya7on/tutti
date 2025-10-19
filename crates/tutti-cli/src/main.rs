use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tutti_config::load_from_path;
use tutti_daemon::DaemonRunner;
use tutti_transport::client::ipc_client::IpcClient;

mod config;

const DEFAULT_FILENAMES: [&str; 3] = ["tutti.toml", "tutti.config.toml", "Tutti.toml"];

// fn string_to_color(s: &str) -> Color {
//     let colors = [
//         Color::Green,
//         Color::Blue,
//         Color::Magenta,
//         Color::Cyan,
//         Color::BrightGreen,
//         Color::BrightBlue,
//         Color::BrightMagenta,
//         Color::BrightCyan,
//     ];

//     let mut hasher = DefaultHasher::new();
//     s.hash(&mut hasher);
//     let hash = hasher.finish();

//     let idx = usize::try_from(hash).unwrap_or_default() % colors.len();
//     colors[idx]
// }

#[tokio::main]
async fn main() -> Result<()> {
    let cli = config::Cli::parse();

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

            let daemon_runner = DaemonRunner::new(system_directory.as_ref().map(PathBuf::from));
            if daemon_runner.prepare().is_err() {
                println!("Failed to prepare daemon");
                return Ok(());
            }

            let path = PathBuf::from(file);
            if !IpcClient::check_socket(&path).await && daemon_runner.spawn().is_err() {
                println!("Failed to spawn daemon");
            }

            let project = load_from_path(&path)?;

            let Ok(mut client) = IpcClient::new(path).await else {
                println!("Failed to connect to the daemon");
                return Ok(());
            };

            if client.up(project, services).await.is_err() {
                println!("Failed to start project");
            }

            let Ok(mut logs) = client.subscribe().await else {
                println!("Failed to subscribe to logs");
                return Ok(());
            };

            while let Some(log) = logs.recv().await {
                println!("{log:?}");
            }

            println!("{:?}", 2);
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
