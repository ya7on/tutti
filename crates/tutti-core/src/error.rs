use tutti_types::ProjectId;

pub type Result<R, E = Error> = std::result::Result<R, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(String),
    #[error("IO error: {0}")]
    IO(std::io::Error),
    #[error("wait error")]
    Wait,
    #[error("project {0} not found")]
    ProjectNotFound(ProjectId),
    #[error("service {1} not found in project {0}")]
    ServiceNotFound(ProjectId, String),
    #[error("circular dependency detected")]
    CircularDependencyDetected,
}
