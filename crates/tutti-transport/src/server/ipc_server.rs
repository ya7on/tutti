use std::{fmt::Debug, path::PathBuf, sync::Arc};

use bytes::Bytes;
use futures_util::{future::BoxFuture, FutureExt, SinkExt, StreamExt};
use tokio::net::UnixListener;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    api::{MessageType, TuttiApi, TuttiMessage},
    error::TransportResult,
};

async fn default_handler(_: TuttiApi) -> TransportResult<TuttiApi> {
    todo!()
}

type Handler = Arc<dyn Fn(TuttiApi) -> BoxFuture<'static, TransportResult<TuttiApi>> + Send + Sync>;

pub struct IpcServer {
    socket: UnixListener,
    handler: Handler,
}

impl Debug for IpcServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IpcServer")
            .field("socket", &self.socket)
            .field("handler", &"[fn]")
            .finish()
    }
}

impl IpcServer {
    pub fn new(path: PathBuf) -> Self {
        let socket = UnixListener::bind(path).unwrap();

        Self {
            socket,
            handler: Arc::new(|api: TuttiApi| default_handler(api).boxed()),
        }
    }

    pub fn add_handler(mut self, handler: Handler) -> Self {
        self.handler = handler;
        self
    }

    pub async fn start(self) {
        while let Ok((stream, _)) = self.socket.accept().await {
            let framed = Framed::new(stream, LengthDelimitedCodec::new());
            let (mut sink, mut stream) = framed.split();
            let handler = self.handler.clone();

            tokio::spawn(async move {
                while let Some(Ok(body)) = stream.next().await {
                    let message = serde_json::from_slice::<TuttiMessage>(&body).unwrap();
                    let response = (handler)(message.body).await.unwrap();

                    let full_response = TuttiMessage {
                        id: message.id,
                        req_type: MessageType::Response,
                        body: response,
                    };
                    let serialized_response = serde_json::to_vec(&full_response).unwrap();

                    sink.send(Bytes::from_iter(serialized_response))
                        .await
                        .unwrap();
                }
            });
        }
    }
}
