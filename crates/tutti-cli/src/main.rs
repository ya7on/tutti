use anyhow::Result;
use clap::Parser;
use tutti_config::load_from_path;
use tutti_core::{LogEvent, Runner, UnixProcessManager};

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = config::Cli::parse();

    match cli.command {
        config::Commands::Run { file } => {
            let path = std::path::Path::new(&file);
            let project = load_from_path(path)?;
            let process_manager = UnixProcessManager::new();
            let mut runner = Runner::new(project, process_manager);

            let mut logs = runner.up().await?;

            tokio::spawn(async move {
                while let Some(log) = logs.recv().await {
                    match log {
                        LogEvent::Log { service_name, line } => {
                            let string = String::from_utf8_lossy(&line);
                            print!("[{service_name}] {string}");
                        }
                        LogEvent::Stop { service_name } => {
                            println!("{service_name} stopped");
                        }
                    }
                }
            });

            runner.wait().await?;
        }
    }

    Ok(())
}
