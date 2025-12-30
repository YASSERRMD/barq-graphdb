use crate::batch_queue::BatchQueue;
use crate::vector::VectorIndex;
use std::sync::Arc;
use std::time::Duration;

pub struct BatchIndexer;

impl BatchIndexer {
    pub fn start_background_thread(
        queue: BatchQueue,
        vector_index: Arc<dyn VectorIndex + Send + Sync>,
        flush_interval: Duration,
    ) {
        std::thread::spawn(move || loop {
            std::thread::sleep(flush_interval);

            let batch = queue.flush();
            let had_items = !batch.is_empty();

            if had_items {
                for node in batch {
                    if !node.embedding.is_empty() {
                        vector_index.insert(node.id, &node.embedding);
                    }
                }
            }

            if queue.is_detached() {
                break;
            }
        });
    }
}
