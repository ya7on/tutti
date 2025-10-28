use std::path::PathBuf;

use futures_util::StreamExt;
use tokio::{
    net::UnixStream,
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tutti_types::{Project, ProjectId};

use crate::{
    api::{MessageType, TuttiApi, TuttiMessage},
    client::worker::IpcClientWorker,
    error::{TransportError, TransportResult},
};

const BUFFER_SIZE: usize = 100;

#[derive(Debug)]
pub struct IpcClient {
    _task: JoinHandle<()>,
    in_socket: mpsc::Sender<(TuttiMessage, mpsc::Sender<TuttiMessage>)>,
    message_counter: u32,
}

impl IpcClient {
    pub async fn check_socket(path: &PathBuf) -> bool {
        let Ok(_socket) = UnixStream::connect(path).await else {
            return false;
        };

        true
    }

    /// Create a new IPC client.
    ///
    /// # Errors
    /// If the socket connection fails.
    ///
    /// # Panics
    /// If the socket connection fails.
    #[tracing::instrument]
    pub async fn new(path: PathBuf) -> TransportResult<Self> {
        let socket = UnixStream::connect(path).await.map_err(|err| {
            tracing::error!("Failed to connect to IPC socket: {}", err);
            TransportError::SocketError(err)
        })?;

        let (tx, rx) = mpsc::channel::<(TuttiMessage, mpsc::Sender<TuttiMessage>)>(BUFFER_SIZE);

        let task = tokio::spawn(async move {
            let framed = Framed::new(socket, LengthDelimitedCodec::new());
            let (sink, stream) = framed.split();
            if IpcClientWorker::new(sink, stream, rx).run().await.is_err() {
                todo!()
            }
        });

        Ok(Self {
            _task: task,
            in_socket: tx,
            message_counter: 0,
        })
    }

    /// Send a message to the server.
    ///
    /// # Errors
    /// If the message could not be sent.
    pub async fn send(&mut self, message: TuttiApi) -> TransportResult<TuttiApi> {
        self.message_counter += 1;
        let message_id = self.message_counter;

        let (response_tx, mut response_rx) = mpsc::channel::<TuttiMessage>(BUFFER_SIZE);

        self.in_socket
            .send((
                TuttiMessage {
                    id: message_id,
                    req_type: MessageType::Request,
                    body: message,
                },
                response_tx,
            ))
            .await
            .map_err(|err| TransportError::SendError(err.to_string()))?;

        while let Some(response) = response_rx.recv().await {
            tracing::debug!("Getting message from rx");

            if let TuttiMessage {
                id,
                req_type: MessageType::Response,
                body,
            } = response
            {
                if id != message_id {
                    continue;
                }

                return Ok(body);
            }
        }

        Err(TransportError::SendError("No response".to_string()))
    }

    pub async fn ping(&mut self) -> bool {
        self.send(TuttiApi::Ping).await.is_ok()
    }

    /// Start a project with the given services.
    ///
    /// # Errors
    /// Returns an error if the project cannot be started.
    pub async fn up(&mut self, project: Project, services: Vec<String>) -> TransportResult<()> {
        tracing::debug!("Starting services");

        self.send(TuttiApi::Up { project, services }).await?;

        Ok(())
    }

    /// Stop a project.
    ///
    /// # Errors
    /// Returns an error if the project cannot be stopped.
    pub async fn down(&mut self, project_id: ProjectId) -> TransportResult<()> {
        tracing::debug!("Stopping services");

        self.send(TuttiApi::Down { project_id }).await?;

        Ok(())
    }

    /// Stop a project.
    ///
    /// # Errors
    /// Returns an error if the project cannot be stopped.
    pub async fn shutdown(&mut self) -> TransportResult<()> {
        tracing::debug!("Stopping services");

        self.message_counter += 1;
        let message_id = self.message_counter;

        let (response_tx, _response_rx) = mpsc::channel::<TuttiMessage>(BUFFER_SIZE);

        self.in_socket
            .send((
                TuttiMessage {
                    id: message_id,
                    req_type: MessageType::Request,
                    body: TuttiApi::Shutdown,
                },
                response_tx,
            ))
            .await
            .map_err(|err| TransportError::SendError(err.to_string()))?;

        Ok(())
    }

    /// Subscribe to Tutti events.
    ///
    /// # Errors
    /// Returns an error if the subscription cannot be established.
    pub async fn subscribe(&mut self) -> TransportResult<Receiver<TuttiMessage>> {
        let (response_tx, stream) = mpsc::channel::<TuttiMessage>(BUFFER_SIZE);

        self.in_socket
            .send((
                TuttiMessage {
                    id: 0,
                    req_type: MessageType::Request,
                    body: TuttiApi::Subscribe,
                },
                response_tx,
            ))
            .await
            .map_err(|err| TransportError::SendError(err.to_string()))?;

        Ok(stream)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tokio::task;

    fn new_client() -> (
        IpcClient,
        Receiver<(TuttiMessage, mpsc::Sender<TuttiMessage>)>,
    ) {
        let (tx, rx) = mpsc::channel::<(TuttiMessage, mpsc::Sender<TuttiMessage>)>(BUFFER_SIZE);
        (
            IpcClient {
                _task: tokio::spawn(async move {}),
                in_socket: tx,
                message_counter: 0,
            },
            rx,
        )
    }

    #[tokio::test]
    async fn test_send() {
        let (mut client, mut rx) = new_client();

        let server = task::spawn(async move {
            let (req, resp_tx) = rx.recv().await.expect("request");
            assert_eq!(req.id, 1);
            assert!(matches!(req.req_type, MessageType::Request));
            assert!(matches!(req.body, TuttiApi::Ping));

            let response = TuttiMessage {
                id: req.id,
                req_type: MessageType::Response,
                body: TuttiApi::Ping,
            };
            resp_tx.send(response).await.unwrap();
        });

        let res = client.send(TuttiApi::Ping).await;
        server.await.unwrap();

        assert!(matches!(res, Ok(TuttiApi::Ping)));
    }

    #[tokio::test]
    async fn test_send_ignore_mismatched_ids() {
        let (mut client, mut rx) = new_client();

        let server = task::spawn(async move {
            let (req, resp_tx) = rx.recv().await.expect("request");

            let wrong = TuttiMessage {
                id: req.id + 42,
                req_type: MessageType::Response,
                body: TuttiApi::Ping,
            };
            resp_tx.send(wrong).await.unwrap();

            let ok = TuttiMessage {
                id: req.id,
                req_type: MessageType::Response,
                body: TuttiApi::Ping,
            };
            resp_tx.send(ok).await.unwrap();
        });

        let res = client.send(TuttiApi::Ping).await;
        server.await.unwrap();

        assert!(matches!(res, Ok(TuttiApi::Ping)));
    }

    #[tokio::test]
    async fn test_ping() {
        let (mut client, mut rx) = new_client();

        let server = task::spawn(async move {
            let (req, resp_tx) = rx.recv().await.expect("request");
            assert_eq!(req.id, 1);
            assert!(matches!(req.req_type, MessageType::Request));
            assert!(matches!(req.body, TuttiApi::Ping));

            let response = TuttiMessage {
                id: req.id,
                req_type: MessageType::Response,
                body: TuttiApi::Ping,
            };
            resp_tx.send(response).await.unwrap();
        });

        let res = client.ping().await;
        server.await.unwrap();

        assert!(res);
    }
}
