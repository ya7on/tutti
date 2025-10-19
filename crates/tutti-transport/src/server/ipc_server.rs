use std::{fmt::Debug, path::PathBuf, sync::Arc};

use bytes::Bytes;
use futures_util::{future::BoxFuture, SinkExt, StreamExt};
use tokio::{
    net::UnixListener,
    sync::{mpsc, RwLock},
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    api::{MessageType, TuttiApi, TuttiMessage},
    error::TransportResult,
    server::fanout::Fanout,
};

type UnaryHandler<C> =
    Arc<dyn Fn(TuttiApi, C) -> BoxFuture<'static, TransportResult<TuttiApi>> + Send + Sync>;

type StreamHandler<C> =
    Arc<dyn Fn(C) -> BoxFuture<'static, TransportResult<TuttiApi>> + Send + Sync>;

pub struct IpcServer<C: Clone + Send + Sync> {
    socket: UnixListener,
    unary_handler: UnaryHandler<C>,
    stream_handler: StreamHandler<C>,
    context: C,
    fanout: Arc<RwLock<Fanout<TuttiMessage>>>,
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
            unary_handler: Arc::new(|_api: TuttiApi, _context: C| unimplemented!()),
            stream_handler: Arc::new(|_context: C| unimplemented!()),
            context,
            fanout: Arc::new(RwLock::new(Fanout::new())),
        }
    }

    pub fn add_unary_handler(mut self, handler: UnaryHandler<C>) -> Self {
        self.unary_handler = handler;
        self
    }

    pub fn add_stream_handler(mut self, handler: StreamHandler<C>) -> Self {
        self.stream_handler = handler;
        self
    }

    pub async fn start(self) {
        let fanout = self.fanout.clone();

        let stream_handler = self.stream_handler.clone();
        let context = self.context.clone();
        let fanout_clone = fanout.clone();
        tokio::spawn(async move {
            loop {
                let message = stream_handler(context.clone()).await.unwrap();
                let guard = fanout_clone.read().await;

                let full_response = TuttiMessage {
                    id: 0,
                    req_type: MessageType::Stream,
                    body: message,
                };
                guard.send(full_response).await;
            }
        });

        while let Ok((stream, _)) = self.socket.accept().await {
            let framed = Framed::new(stream, LengthDelimitedCodec::new());
            let (mut sink, mut stream) = framed.split();

            let (tx, mut rx) = mpsc::channel::<TuttiMessage>(10);

            {
                let tx_clone = tx.clone();
                let mut guard = fanout.write().await;
                guard.subscribe(tx_clone);
            }

            let unary_handler = self.unary_handler.clone();
            let context = self.context.clone();
            tokio::spawn(async move {
                while let Some(Ok(body)) = stream.next().await {
                    let message = serde_json::from_slice::<TuttiMessage>(&body).unwrap();
                    let response = (unary_handler)(message.body, context.clone())
                        .await
                        .unwrap();

                    let full_response = TuttiMessage {
                        id: message.id,
                        req_type: MessageType::Response,
                        body: response,
                    };
                    tx.send(full_response).await.unwrap();
                }
            });

            while let Some(message) = rx.recv().await {
                let serialized_response = serde_json::to_vec(&message).unwrap();
                sink.send(Bytes::from_iter(serialized_response))
                    .await
                    .unwrap();
            }
        }
    }
}
