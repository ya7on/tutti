use std::path::PathBuf;

use futures_util::StreamExt;
use tokio::{net::UnixStream, sync::mpsc, task::JoinHandle};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    api::{MessageType, TuttiApi, TuttiMessage},
    client::worker::IpcClientWorker,
};

const BUFFER_SIZE: usize = 100;

#[derive(Debug)]
pub struct IpcClient {
    _task: JoinHandle<()>,
    in_socket: mpsc::Sender<(TuttiMessage, mpsc::Sender<TuttiMessage>)>,
    message_counter: u32,
}

impl IpcClient {
    pub async fn new(path: PathBuf) -> Self {
        let socket = UnixStream::connect(path).await.unwrap();

        let (tx, rx) = mpsc::channel::<(TuttiMessage, mpsc::Sender<TuttiMessage>)>(BUFFER_SIZE);

        let task = tokio::spawn(async move {
            let framed = Framed::new(socket, LengthDelimitedCodec::new());
            let (sink, stream) = framed.split();
            IpcClientWorker::new(sink, stream, rx).run().await.unwrap();
        });

        Self {
            _task: task,
            in_socket: tx,
            message_counter: 0,
        }
    }

    pub async fn send(&mut self, message: TuttiApi) -> Result<TuttiApi, ()> {
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
            .unwrap();

        while let Some(response) = response_rx.recv().await {
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
        Err(())
    }
}
