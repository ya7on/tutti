use std::{fmt::Debug, path::PathBuf, sync::Arc};

use bytes::Bytes;
use futures_util::{future::BoxFuture, FutureExt, SinkExt, StreamExt};
use tokio::net::UnixListener;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    api::{MessageType, TuttiApi, TuttiMessage},
    error::TransportResult,
};

async fn default_handler<C: Clone + Send + Sync + 'static>(
    _: TuttiApi,
    _: C,
) -> TransportResult<TuttiApi> {
    todo!()
}

type Handler<C: Clone + Send + Sync + 'static> =
    Arc<dyn Fn(TuttiApi, C) -> BoxFuture<'static, TransportResult<TuttiApi>> + Send + Sync>;

pub struct IpcServer<C: Clone + Send + Sync> {
    socket: UnixListener,
    handler: Handler<C>,
    context: C,
}

impl<C: Clone + Debug + Send + Sync + 'static> Debug for IpcServer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IpcServer")
            .field("socket", &self.socket)
            .field("context", &self.context)
            .field("handler", &"[fn]")
            .finish()
    }
}

impl<C: Clone + Debug + Send + Sync + 'static> IpcServer<C> {
    pub fn new(path: PathBuf, context: C) -> Self {
        let socket = UnixListener::bind(path).unwrap();

        Self {
            socket,
            handler: Arc::new(|api: TuttiApi, context: C| {
                default_handler::<C>(api, context).boxed()
            }),
            context,
        }
    }

    pub fn add_handler(mut self, handler: Handler<C>) -> Self {
        self.handler = handler;
        self
    }

    pub async fn start(self) {
        while let Ok((stream, _)) = self.socket.accept().await {
            let framed = Framed::new(stream, LengthDelimitedCodec::new());
            let (mut sink, mut stream) = framed.split();
            let handler = self.handler.clone();
            let context = self.context.clone();

            tokio::spawn(async move {
                while let Some(Ok(body)) = stream.next().await {
                    let message = serde_json::from_slice::<TuttiMessage>(&body).unwrap();
                    let response = (handler)(message.body, context.clone()).await.unwrap();

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
