use std::collections::HashMap;

use bytes::{Bytes, BytesMut};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{net::UnixStream, select, sync::mpsc};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{api::TuttiMessage, error::TransportResult};

#[derive(Debug)]
pub struct IpcClientWorker {
    sink: SplitSink<Framed<UnixStream, LengthDelimitedCodec>, Bytes>,
    stream: SplitStream<Framed<UnixStream, LengthDelimitedCodec>>,

    // socket: Framed<UnixStream, LengthDelimitedCodec>,
    receiver: mpsc::Receiver<(TuttiMessage, mpsc::Sender<TuttiMessage>)>,

    response: HashMap<u32, mpsc::Sender<TuttiMessage>>,
}

impl IpcClientWorker {
    pub fn new(
        socket: Framed<UnixStream, LengthDelimitedCodec>,
        receiver: mpsc::Receiver<(TuttiMessage, mpsc::Sender<TuttiMessage>)>,
    ) -> Self {
        let (sink, stream) = socket.split();

        Self {
            sink,
            stream,
            receiver,
            response: HashMap::new(),
        }
    }

    async fn handle_socket_message(&mut self, message: BytesMut) -> TransportResult<()> {
        let message = serde_json::from_slice::<TuttiMessage>(&message).unwrap();

        if let Some(response) = self.response.get(&message.id) {
            response.send(message).await.unwrap();
        }

        todo!()
    }

    async fn handle_mpsc_message(
        &mut self,
        message: TuttiMessage,
        sender: mpsc::Sender<TuttiMessage>,
    ) -> TransportResult<()> {
        let message_id = message.id;

        let b = serde_json::to_vec(&message).unwrap();
        self.sink.send(Bytes::from(b)).await.unwrap();

        self.response.insert(message_id, sender);

        todo!()
    }

    pub async fn run(&mut self) -> TransportResult<()> {
        loop {
            select! {
                Some(Ok(msg)) = self.stream.next() => {
                    self.handle_socket_message(msg).await?
                }
                Some((msg, sender)) = self.receiver.recv() => {
                    self.handle_mpsc_message(msg, sender).await?
                }
            }
        }
    }
}
