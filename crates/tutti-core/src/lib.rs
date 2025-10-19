mod error;
mod process_manager;
mod supervisor;

#[cfg(unix)]
pub use process_manager::UnixProcessManager;
pub use process_manager::{CommandSpec, ProcId, ProcessManager, Spawned};
pub use supervisor::{Supervisor, SupervisorEvent, UpResponse};
