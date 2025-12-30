use dashmap::DashMap;
use hnsw_rs::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::VectorIndex;
use crate::NodeId;

/// HNSW-based vector index implementation.
/// Uses logical-to-physical ID mapping to support updates via append-only strategy.
/// Thread-safe implementation using DashMap and AtomicUsize.
pub struct HnswVectorIndex {
    /// The underlying HNSW index (thread-safe).
    index: Hnsw<'static, f32, DistL2>,
    /// Maps NodeId (logical) to the current valid Internal ID (physical) in HNSW.
    node_to_internal: DashMap<NodeId, usize>,
    /// Maps Internal ID (physical) back to NodeId (logical).
    internal_to_node: DashMap<usize, NodeId>,
    /// Counter for assigning new internal IDs.
    next_internal_id: AtomicUsize,
}

impl HnswVectorIndex {
    /// Creates a new HNSW index.
    pub fn new(max_elements: usize) -> Self {
        let max_nb_connection = 16; // M
        let ef_construction = 200; // build quality

        let index = Hnsw::new(
            max_nb_connection,
            max_elements,
            16, // max_layer
            ef_construction,
            DistL2 {},
        );

        Self {
            index,
            node_to_internal: DashMap::new(),
            internal_to_node: DashMap::new(),
            next_internal_id: AtomicUsize::new(1),
        }
    }
}

impl VectorIndex for HnswVectorIndex {
    fn insert(&self, id: NodeId, embedding: &[f32]) {
        // Assign a new internal ID atomically
        // Relaxed ordering is fine as unique IDs matters, strict time ordering is loose in distrib DBs,
        // but SeqCst is safer for logic if needed. Relaxed is enough for counter.
        let internal_id = self.next_internal_id.fetch_add(1, Ordering::Relaxed);

        // Insert into HNSW (internal locking)
        self.index.insert((embedding, internal_id));

        // Update mappings (DashMap handles concurrency)
        self.node_to_internal.insert(id, internal_id);
        self.internal_to_node.insert(internal_id, id);
    }

    fn knn(&self, query: &[f32], k: usize) -> Vec<(NodeId, f32)> {
        let ef_search = 50.max(k);
        let fetch_k = k * 5;

        // HNSW search is thread-safe
        let results = self.index.search(query, fetch_k, ef_search);

        let mut final_results = Vec::with_capacity(k);
        // We use a small local set to dedup results for this query
        let mut seen_nodes = std::collections::HashSet::new();

        for neighbor in results {
            let internal_id = neighbor.d_id;

            // Resolve logical ID using concurrent map
            if let Some(node_ref) = self.internal_to_node.get(&internal_id) {
                let node_id = *node_ref.value();

                // Check if this internal ID is CURRENT
                if let Some(current_ref) = self.node_to_internal.get(&node_id) {
                    if *current_ref.value() == internal_id {
                        // It's valid!
                        if seen_nodes.insert(node_id) {
                            final_results.push((node_id, neighbor.distance));
                            if final_results.len() >= k {
                                break;
                            }
                        }
                    }
                }
            }
        }

        final_results
    }

    fn len(&self) -> usize {
        self.node_to_internal.len()
    }

    fn contains(&self, id: NodeId) -> bool {
        self.node_to_internal.contains_key(&id)
    }
}
