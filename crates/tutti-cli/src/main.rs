use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use tutti_config::load_from_path;
use tutti_core::{CommandSpec, ProcessManager, UnixProcessManager};

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = config::Cli::parse();

    let mut process_manager = UnixProcessManager::new();
    let mut running = Vec::new();

    match cli.command {
        config::Commands::Run { file } => {
            let path = std::path::Path::new(&file);
            let project = load_from_path(path)?;

            for (key, service) in project.services {
                println!("Running service {key}");

                let mut spawned = process_manager
                    .spawn(CommandSpec {
                        name: key.clone(),
                        cmd: service.cmd,
                        cwd: None,   // TODO
                        env: vec![], // TODO
                    })
                    .await?;
                let task = tokio::spawn(async move {
                    while let Some(line) = spawned.stdout.next().await {
                        let s = String::from_utf8(line).unwrap_or_default();
                        print!("[{key}] {s}");
                    }
                });
                running.push(task);
            }

            for task in running {
                task.await?;
            }
        }
    }

    Ok(())
}
