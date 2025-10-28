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

        /// System directory path
        #[arg(short, long)]
        system_directory: Option<String>,

        /// Timeout for killing services (in seconds)
        #[arg(short, long)]
        kill_timeout: Option<u64>,
    },
    /// Manage tutti daemon service
    Daemon {
        #[command(subcommand)]
        cmd: DaemonCmd,

        /// System directory path
        #[arg(short, long)]
        system_directory: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum DaemonCmd {
    /// Start the daemon service
    Run,
    /// Stop the daemon service
    Stop,
}
