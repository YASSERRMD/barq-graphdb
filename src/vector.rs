//! Vector index for similarity search.
//!
//! This module provides vector indexing and k-nearest neighbor (kNN) search
//! functionality using L2 (Euclidean) distance.

use std::collections::HashMap;

use crate::NodeId;

/// Trait for vector index implementations.
///
/// Different implementations can provide various trade-offs between
/// speed, memory usage, and accuracy.
pub trait VectorIndex: Send + Sync {
    /// Inserts a vector embedding for a node.
    ///
    /// # Arguments
    ///
    /// * `id` - Node ID associated with this embedding
    /// * `embedding` - Vector embedding to store
    fn insert(&mut self, id: NodeId, embedding: &[f32]);

    /// Finds the k nearest neighbors to a query vector.
    ///
    /// # Arguments
    ///
    /// * `query` - Query vector for similarity search
    /// * `k` - Number of nearest neighbors to return
    ///
    /// # Returns
    ///
    /// A vector of (NodeId, distance) pairs sorted by distance ascending.
    fn knn(&self, query: &[f32], k: usize) -> Vec<(NodeId, f32)>;

    /// Returns the number of vectors in the index.
    fn len(&self) -> usize;

    /// Returns true if the index is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if a node exists in the index.
    fn contains(&self, id: NodeId) -> bool;

    /// Gets the embedding for a node if it exists.
    fn get(&self, id: NodeId) -> Option<&[f32]>;
}

/// Computes the L2 (Euclidean) distance between two vectors.
///
/// # Arguments
///
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
///
/// The L2 distance (not squared) between the vectors.
///
/// # Panics
///
/// If vectors have different lengths (in debug mode).
pub fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(
        a.len(),
        b.len(),
        "Vectors must have same length for L2 distance"
    );

    let mut sum = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        let diff = x - y;
        sum += diff * diff;
    }
    sum.sqrt()
}

/// Computes cosine distance between two vectors.
///
/// Cosine distance = 1 - cosine_similarity
///
/// # Arguments
///
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
///
/// The cosine distance (0 = identical, 2 = opposite).
#[allow(dead_code)]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(
        a.len(),
        b.len(),
        "Vectors must have same length for cosine distance"
    );

    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let magnitude = (norm_a * norm_b).sqrt();
    if magnitude == 0.0 {
        return 1.0; // Undefined for zero vectors, return max distance
    }

    1.0 - (dot / magnitude)
}

/// Linear scan vector index implementation.
///
/// This is a simple brute-force implementation that computes
/// distances to all vectors for each query. Suitable for small
/// datasets (< 10,000 vectors).
#[derive(Debug, Default)]
pub struct LinearVectorIndex {
    /// Storage mapping node IDs to their embeddings.
    vectors: HashMap<NodeId, Vec<f32>>,
}

impl LinearVectorIndex {
    /// Creates a new empty linear vector index.
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
        }
    }
}

impl VectorIndex for LinearVectorIndex {
    fn insert(&mut self, id: NodeId, embedding: &[f32]) {
        self.vectors.insert(id, embedding.to_vec());
    }

    fn knn(&self, query: &[f32], k: usize) -> Vec<(NodeId, f32)> {
        // Compute distances to all vectors
        let mut distances: Vec<(NodeId, f32)> = self
            .vectors
            .iter()
            .filter(|(_, vec)| vec.len() == query.len())
            .map(|(&id, vec)| (id, l2_distance(query, vec)))
            .collect();

        // Sort by distance (ascending)
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top k
        distances.truncate(k);
        distances
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }

    fn contains(&self, id: NodeId) -> bool {
        self.vectors.contains_key(&id)
    }

    fn get(&self, id: NodeId) -> Option<&[f32]> {
        self.vectors.get(&id).map(|v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l2_distance_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!((l2_distance(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_l2_distance_simple() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((l2_distance(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_l2_distance_single_dim() {
        let a = vec![0.0];
        let b = vec![5.0];
        assert!((l2_distance(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance_parallel() {
        let a = vec![1.0, 0.0];
        let b = vec![2.0, 0.0];
        assert!((cosine_distance(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_distance(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_index_new() {
        let index = LinearVectorIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_linear_index_insert_and_contains() {
        let mut index = LinearVectorIndex::new();
        index.insert(1, &[0.1, 0.2, 0.3]);

        assert!(index.contains(1));
        assert!(!index.contains(2));
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_linear_index_get() {
        let mut index = LinearVectorIndex::new();
        let embedding = vec![0.1, 0.2, 0.3];
        index.insert(1, &embedding);

        let retrieved = index.get(1).unwrap();
        assert_eq!(retrieved, &embedding);
        assert!(index.get(999).is_none());
    }

    #[test]
    fn test_knn_simple() {
        let mut index = LinearVectorIndex::new();

        // Insert vectors in 2D space
        index.insert(1, &[0.0, 0.0]);
        index.insert(2, &[1.0, 0.0]);
        index.insert(3, &[0.0, 1.0]);
        index.insert(4, &[1.0, 1.0]);
        index.insert(5, &[5.0, 5.0]);

        // Query at origin - nearest should be node 1
        let results = index.knn(&[0.0, 0.0], 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 1); // Closest
        assert!((results[0].1 - 0.0).abs() < 1e-6); // Distance 0

        // Nodes 2 and 3 should be tied at distance 1
        let next_ids: Vec<NodeId> = results[1..].iter().map(|(id, _)| *id).collect();
        assert!(next_ids.contains(&2) || next_ids.contains(&3));
    }

    #[test]
    fn test_knn_k_larger_than_dataset() {
        let mut index = LinearVectorIndex::new();
        index.insert(1, &[0.0]);
        index.insert(2, &[1.0]);

        let results = index.knn(&[0.0], 10);
        assert_eq!(results.len(), 2); // Only 2 vectors exist
    }

    #[test]
    fn test_knn_empty_index() {
        let index = LinearVectorIndex::new();
        let results = index.knn(&[0.0, 0.0], 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_knn_ordering() {
        let mut index = LinearVectorIndex::new();
        index.insert(1, &[0.0]);
        index.insert(2, &[3.0]);
        index.insert(3, &[1.0]);
        index.insert(4, &[2.0]);

        let results = index.knn(&[0.0], 4);

        // Should be ordered by distance: 1, 3, 4, 2
        assert_eq!(results[0].0, 1);
        assert_eq!(results[1].0, 3);
        assert_eq!(results[2].0, 4);
        assert_eq!(results[3].0, 2);

        // Verify distances are sorted
        for i in 0..results.len() - 1 {
            assert!(results[i].1 <= results[i + 1].1);
        }
    }
}
