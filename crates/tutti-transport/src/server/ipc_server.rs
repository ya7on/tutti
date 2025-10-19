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
    error::{TransportError, TransportResult},
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
            .field("fanout", &self.fanout)
            .field("unary_handler", &"[fn]")
            .field("stream_handler", &"[fn]")
            .finish()
    }
}

impl<C: Clone + Debug + Send + Sync + 'static> IpcServer<C> {
    /// Create a new IPC server.
    ///
    /// # Errors
    /// Returns a `TransportError` if the Unix socket cannot be bound.
    pub fn new(path: PathBuf, context: C) -> TransportResult<Self> {
        let socket = UnixListener::bind(path).map_err(TransportError::SocketError)?;

        Ok(Self {
            socket,
            unary_handler: Arc::new(|_api: TuttiApi, _context: C| unimplemented!()),
            stream_handler: Arc::new(|_context: C| unimplemented!()),
            context,
            fanout: Arc::new(RwLock::new(Fanout::new())),
        })
    }

    #[must_use]
    pub fn add_unary_handler(mut self, handler: UnaryHandler<C>) -> Self {
        self.unary_handler = handler;
        self
    }

    #[must_use]
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
                let Ok(message) = stream_handler(context.clone()).await else {
                    continue;
                };
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
                    let Ok(message) = serde_json::from_slice::<TuttiMessage>(&body) else {
                        continue;
                    };
                    let Ok(response) = (unary_handler)(message.body, context.clone()).await else {
                        continue;
                    };

                    let full_response = TuttiMessage {
                        id: message.id,
                        req_type: MessageType::Response,
                        body: response,
                    };
                    let _ = tx.send(full_response).await;
                }
            });

            while let Some(message) = rx.recv().await {
                let Ok(serialized_response) = serde_json::to_vec(&message) else {
                    continue;
                };
                let _ = sink.send(Bytes::from_iter(serialized_response)).await;
            }
        }
    }
}
