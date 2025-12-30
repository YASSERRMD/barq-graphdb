use crate::Node;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct BatchQueue {
    queue: Arc<Mutex<Vec<Node>>>,
    max_batch_size: usize,
}

impl BatchQueue {
    pub fn new(max_batch_size: usize) -> Self {
        BatchQueue {
            queue: Arc::new(Mutex::new(Vec::new())),
            max_batch_size,
        }
    }

    /// Pushes a node to the queue. Returns true if the queue is full (should flush).
    pub fn push(&self, node: Node) -> bool {
        let mut q = self.queue.lock().unwrap();
        q.push(node);
        q.len() >= self.max_batch_size
    }

    /// Flushes all pending nodes and returns them.
    pub fn flush(&self) -> Vec<Node> {
        let mut q = self.queue.lock().unwrap();
        if q.is_empty() {
            return Vec::new(); // Avoid allocation if empty
        }
        std::mem::take(&mut *q)
    }

    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    /// Checks if this queue instance is the only one holding the underlying data.
    /// Used for thread termination.
    pub fn is_detached(&self) -> bool {
        Arc::strong_count(&self.queue) <= 1
    }
}
