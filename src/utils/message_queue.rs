// TODO: write some tests

use super::Endpoint;

type Message = [u64; 8];

/// A message queue which is checked in ipc RETRIEVE, to
/// check for available messages
pub struct MessageQueue {
    // Vector containing held messages. When an entry is consumed,
    // we replace it with a `None` value. When more than half of data
    // is `None`s, we remove the `None` values
    data: Vec<Option<QueueEntry>>,

    // counts the actual `QueueEntry`s held in data
    some_count: usize,
}
impl MessageQueue {
    pub fn new() -> Self {
        Self {
            data: vec![],
            some_count: 0,
        }
    }

    /// insert a message from sender into the queue
    pub fn insert(&mut self, sender: Endpoint, message: Message) {
        self.data.push(Some(QueueEntry { sender, message }));
        self.some_count += 1;
    }

    /// gets the next message, for which the `sender` satisfies `predicate`
    pub fn get(&mut self, predicate: impl Fn(Endpoint) -> bool) -> Option<(Endpoint, Message)> {
        let entry = self
            .data
            .iter_mut()
            .filter(|entry| entry.is_some() && predicate(entry.as_ref().unwrap().sender))
            .next();

        let result = if let Some(entry) = entry {
            assert!(entry.is_some());
            self.some_count -= 1;
            entry.take().map(|entry| (entry.sender, entry.message))
        } else {
            None
        };

        // remove empty element(s) from the end
        // there can be multiple, if we just removed the last element,
        // and before it there were more `None`s
        while !self.data.is_empty() && self.data.last().unwrap().is_none() {
            self.data.pop();
        }

        // if more than half of `data` is `None`s, remove them from `data`
        if self.some_count > self.data.len() / 2 {
            // Rust's std Vector has a method for that; Nice!
            self.data.retain(Option::is_some);
        }

        result
    }
}

struct QueueEntry {
    sender: Endpoint,
    message: Message,
}
