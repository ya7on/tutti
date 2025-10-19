use tutti_types::ProjectId;

pub type Result<R, E = Error> = std::result::Result<R, E>;

#[derive(Debug)]
pub enum Error {
    Internal(String),
    IO(std::io::Error),
    Wait,
    ProjectNotFound(ProjectId),
    ServiceNotFound(ProjectId, String),
    CircularDependencyDetected,
}
