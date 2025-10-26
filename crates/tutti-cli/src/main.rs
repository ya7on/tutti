use anyhow::Result;
use clap::Parser;

use crate::commands::{daemon, run};

mod commands;
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
            kill_timeout,
        } => run(file, services, system_directory, kill_timeout).await?,
        config::Commands::Daemon { system_directory } => daemon(system_directory).await?,
    }

    Ok(())
}
