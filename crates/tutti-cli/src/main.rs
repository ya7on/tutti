use anyhow::Result;
use clap::Parser;
use tutti_config::load_from_path;
use tutti_core::{CommandSpec, ProcessManager, UnixProcessManager};

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = config::Cli::parse();

    let mut process_manager = UnixProcessManager::new();

    match cli.command {
        config::Commands::Run { file } => {
            let path = std::path::Path::new(&file);
            let project = load_from_path(path).expect("Failed to load project");

            for (key, service) in project.services {
                println!("Running service {}", key);

                process_manager
                    .spawn(CommandSpec {
                        name: key,
                        cmd: service.cmd,
                        cwd: None,   // TODO
                        env: vec![], // TODO
                    })
                    .await
                    .expect("Failed to start service");
            }
        }
    }

    Ok(())
}
