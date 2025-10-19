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
            if self.subscribers[i].try_send(message.clone()).is_err() {
                todo!()
            }
            let _ = self.subscribers[i].send(message.clone()).await;
        }
    }
}
