use anyhow::Result;
use clap::Parser;
use tutti_config::load_from_path;
use tutti_core::{Runner, UnixProcessManager};

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
                    let string = String::from_utf8_lossy(&log);
                    println!("{string}");
                }
            });

            runner.wait().await?;
        }
    }

    Ok(())
}
