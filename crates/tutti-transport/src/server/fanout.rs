use tokio::sync::mpsc::Sender;

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

    pub async fn send(&mut self, message: T) {
        for i in 0..self.subscribers.len() {
            if let Err(_) = self.subscribers[i].try_send(message.clone()) {
                self.subscribers.swap_remove(i);
            }
        }
    }
}
