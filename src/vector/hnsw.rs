use std::collections::{HashMap, HashSet};
use hnsw_rs::prelude::*;
use crate::NodeId;
use super::VectorIndex;

/// HNSW-based vector index implementation.
/// Uses logical-to-physical ID mapping to support updates via append-only strategy.
pub struct HnswVectorIndex {
    /// The underlying HNSW index.
    index: Hnsw<'static, f32, DistL2>,
    /// Maps NodeId (logical) to the current valid Internal ID (physical) in HNSW.
    node_to_internal: HashMap<NodeId, usize>,
    /// Maps Internal ID (physical) back to NodeId (logical).
    internal_to_node: HashMap<usize, NodeId>,
    /// Counter for assigning new internal IDs.
    next_internal_id: usize,
}

impl HnswVectorIndex {
    /// Creates a new HNSW index.
    pub fn new(max_elements: usize) -> Self {
        let max_nb_connection = 16; // M
        let ef_construction = 200;  // build quality
        
        // HNSW configuration
        // Note: max_elements here is for capacity estimation basically
        let index = Hnsw::new(
            max_nb_connection, 
            max_elements, 
            16, // max_layer
            ef_construction,
            DistL2 {}
        );
        
        Self {
            index,
            node_to_internal: HashMap::new(),
            internal_to_node: HashMap::new(),
            next_internal_id: 1, // Start from 1
        }
    }
}

impl VectorIndex for HnswVectorIndex {
    fn insert(&mut self, id: NodeId, embedding: &[f32]) {
        // Assign a new internal ID for this version of the vector
        let internal_id = self.next_internal_id;
        self.next_internal_id += 1;

        // Insert into HNSW with the new internal ID
        self.index.insert((embedding, internal_id));

        // Update mappings
        self.node_to_internal.insert(id, internal_id);
        self.internal_to_node.insert(internal_id, id);
    }

    fn knn(&self, query: &[f32], k: usize) -> Vec<(NodeId, f32)> {
        let ef_search = 50.max(k);
        let fetch_k = k * 5; // Fetch more candidates to account for stale updates
        
        let results = self.index.search(query, fetch_k, ef_search);
        
        let mut final_results = Vec::with_capacity(k);
        let mut seen_nodes = HashSet::new();

        for neighbor in results {
            let internal_id = neighbor.d_id;
            
            // Resolve logical ID
            if let Some(&node_id) = self.internal_to_node.get(&internal_id) {
                // Check if this internal ID is the CURRENT one for this node
                if let Some(&current_internal) = self.node_to_internal.get(&node_id) {
                    if current_internal == internal_id {
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
