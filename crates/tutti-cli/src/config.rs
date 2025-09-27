use clap::{Parser, Subcommand};

/// CLI для tutti
#[derive(Parser, Debug)]
#[command(name = "tutti", version, about = "Local service orchestrator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Запускает проект по указанному конфигу
    Run {
        /// Путь к конфиг-файлу (TOML/YAML)
        #[arg(short, long)]
        file: String,
    },
}
