use clap::{Parser, Subcommand};

/// CLI for tutti
#[derive(Parser, Debug)]
#[command(name = "tutti", version, about = "Local service orchestrator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the project using the specified configuration
    Run {
        /// File path to the configuration file (TOML)
        #[arg(short, long)]
        file: Option<String>,

        /// Services to start
        services: Vec<String>,
    },
}
