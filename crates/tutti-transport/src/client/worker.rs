use std::collections::HashMap;

use bytes::{Bytes, BytesMut};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::UnixStream,
    select,
    sync::mpsc,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    api::{TuttiApi, TuttiMessage},
    error::TransportResult,
};

pub type IpcWorkerSink<IO = UnixStream> = SplitSink<Framed<IO, LengthDelimitedCodec>, Bytes>;
pub type IpcWorkerStream<IO = UnixStream> = SplitStream<Framed<IO, LengthDelimitedCodec>>;

#[derive(Debug)]
pub struct IpcClientWorker<IO = UnixStream> {
    sink: IpcWorkerSink<IO>,
    stream: IpcWorkerStream<IO>,

    receiver: mpsc::Receiver<(TuttiMessage, mpsc::Sender<TuttiMessage>)>,
    streams: Vec<mpsc::Sender<TuttiMessage>>,

    response: HashMap<u32, mpsc::Sender<TuttiMessage>>,
}

impl<IO> IpcClientWorker<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    pub fn new(
        sink: IpcWorkerSink<IO>,
        stream: IpcWorkerStream<IO>,
        receiver: mpsc::Receiver<(TuttiMessage, mpsc::Sender<TuttiMessage>)>,
    ) -> Self {
        Self {
            sink,
            stream,
            receiver,
            streams: Vec::new(),
            response: HashMap::new(),
        }
    }

    async fn handle_socket_message(&mut self, message: BytesMut) -> TransportResult<()> {
        let message = serde_json::from_slice::<TuttiMessage>(&message).unwrap();

        if message.id == 0 {
            for stream in &mut self.streams {
                stream.send(message.clone()).await.unwrap();
            }
            return Ok(());
        }

        if let Some(response) = self.response.remove(&message.id) {
            response.send(message).await.unwrap();
        }

        Ok(())
    }

    async fn handle_mpsc_message(
        &mut self,
        message: TuttiMessage,
        sender: mpsc::Sender<TuttiMessage>,
    ) -> TransportResult<()> {
        let message_id = message.id;

        if message_id == 0 && message.body == TuttiApi::Subscribe {
            self.streams.push(sender);
            return Ok(());
        }

        let b = serde_json::to_vec(&message).unwrap();
        self.sink.send(Bytes::from(b)).await.unwrap();

        self.response.insert(message_id, sender);

        Ok(())
    }

    pub async fn run(&mut self) -> TransportResult<()> {
        loop {
            select! {
                Some(Ok(msg)) = self.stream.next() => {
                    self.handle_socket_message(msg).await?;
                }
                Some((msg, sender)) = self.receiver.recv() => {
                    self.handle_mpsc_message(msg, sender).await?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::StreamExt;
    use tokio::{
        io::{duplex, DuplexStream},
        sync::mpsc::error::TryRecvError,
    };

    use crate::api::{MessageType, TuttiApi};

    use super::*;

    struct PrepareWorker {
        worker: IpcClientWorker<DuplexStream>,
        _server_io: DuplexStream,
    }

    async fn prepare_worker() -> PrepareWorker {
        let (client_io, server_io) = duplex(64 * 1024);
        let (_, req_rx) = mpsc::channel::<(TuttiMessage, mpsc::Sender<TuttiMessage>)>(8);

        let framed = Framed::new(client_io, LengthDelimitedCodec::new());
        let (sink, stream) = framed.split();

        let worker = IpcClientWorker::new(sink, stream, req_rx);

        PrepareWorker {
            worker,
            _server_io: server_io,
        }
    }

    #[tokio::test]
    async fn test_worker() {
        let mut fixture = prepare_worker().await;

        let (tx, mut rx) = mpsc::channel::<TuttiMessage>(8);

        fixture
            .worker
            .handle_mpsc_message(
                TuttiMessage {
                    id: 42,
                    req_type: MessageType::Request,
                    body: TuttiApi::Ping,
                },
                tx,
            )
            .await
            .unwrap();
        {
            let message = TuttiMessage {
                id: 69,
                req_type: MessageType::Response,
                body: TuttiApi::Pong,
            };
            let bytes = BytesMut::from_iter(serde_json::to_vec(&message).unwrap());
            fixture.worker.handle_socket_message(bytes).await.unwrap();
        }
        {
            let message = TuttiMessage {
                id: 42,
                req_type: MessageType::Response,
                body: TuttiApi::Pong,
            };
            let bytes = BytesMut::from_iter(serde_json::to_vec(&message).unwrap());
            fixture.worker.handle_socket_message(bytes).await.unwrap();
        }

        let response = rx.recv().await.unwrap();
        assert_eq!(response.id, 42);
        assert_eq!(response.req_type, MessageType::Response);
        assert_eq!(response.body, TuttiApi::Pong);

        let response = rx.try_recv();
        assert!(response.is_err());
        assert_eq!(response.err().unwrap(), TryRecvError::Disconnected);
    }
}
