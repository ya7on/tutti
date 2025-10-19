pub type TransportResult<T, E = TransportError> = Result<T, E>;

#[derive(Debug)]
pub enum TransportError {
    SerdeError(serde_json::Error),
    UnknownMessage,
    SocketError(std::io::Error),
    SendError(String),
}
