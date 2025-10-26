use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Fanout<T: Clone> {
    subscribers: Vec<Sender<T>>,
}

impl<T: Clone> Default for Fanout<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Fanout<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, sender: Sender<T>) {
        self.subscribers.push(sender);
    }

    pub async fn send(&self, message: T) {
        for i in 0..self.subscribers.len() {
            // if self.subscribers[i].try_send(message.clone()).is_err() {
            //     todo!()
            // }
            let _ = self.subscribers[i].send(message.clone()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_send_on_empty_does_not_panic() {
        let fanout = Fanout::<String>::new();
        fanout.send("msg".to_string()).await;
    }

    #[tokio::test]
    async fn test_default_works() {
        let mut fanout = Fanout::default();
        let (tx, mut rx) = mpsc::channel(8);
        fanout.subscribe(tx);
        fanout.send("hello".to_string()).await;
        let got = rx.recv().await.unwrap();
        assert_eq!(got, "hello");
    }

    #[tokio::test]
    async fn test_send_to_all_subscribers() {
        let mut fanout = Fanout::new();
        let (tx1, mut rx1) = mpsc::channel(8);
        let (tx2, mut rx2) = mpsc::channel(8);
        let (tx3, mut rx3) = mpsc::channel(8);
        fanout.subscribe(tx1);
        fanout.subscribe(tx2);
        fanout.subscribe(tx3);

        fanout.send(42).await;

        let a = rx1.recv().await.unwrap();
        let b = rx2.recv().await.unwrap();
        let c = rx3.recv().await.unwrap();
        assert_eq!(a, 42);
        assert_eq!(b, 42);
        assert_eq!(c, 42);
    }

    #[tokio::test]
    async fn test_send_with_some_receivers_closed() {
        let mut fanout = Fanout::new();
        let (tx1, mut rx1) = mpsc::channel(8);
        let (tx2, mut rx2) = mpsc::channel(8);
        let (tx3, _rx3) = mpsc::channel::<&'static str>(8);
        drop(_rx3);

        fanout.subscribe(tx1);
        fanout.subscribe(tx2);
        fanout.subscribe(tx3);

        fanout.send("ping").await;

        let a = rx1.recv().await.unwrap();
        let b = rx2.recv().await.unwrap();
        assert_eq!(a, "ping");
        assert_eq!(b, "ping");
    }

    #[tokio::test]
    async fn test_back_to_back_sends_order_preserved_per_subscriber() {
        let mut fanout = Fanout::new();
        let (tx, mut rx) = mpsc::channel(8);
        fanout.subscribe(tx);

        fanout.send("first").await;
        fanout.send("second").await;
        fanout.send("third").await;

        let a = rx.recv().await.unwrap();
        let b = rx.recv().await.unwrap();
        let c = rx.recv().await.unwrap();
        assert_eq!(a, "first");
        assert_eq!(b, "second");
        assert_eq!(c, "third");
    }
}
