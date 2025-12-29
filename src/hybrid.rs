//! Hybrid query combining vector similarity and graph distance.
//!
//! This module provides hybrid scoring that combines vector embedding
//! similarity with graph traversal distance for ranking results.

use crate::NodeId;

/// Parameters for hybrid scoring.
#[derive(Debug, Clone)]
pub struct HybridParams {
    /// Weight for vector similarity component (0.0 to 1.0).
    pub alpha: f32,
    /// Weight for graph distance component (0.0 to 1.0).
    pub beta: f32,
}

impl Default for HybridParams {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            beta: 0.5,
        }
    }
}

impl HybridParams {
    /// Creates new hybrid parameters with specified weights.
    ///
    /// # Arguments
    ///
    /// * `alpha` - Weight for vector similarity (higher = more emphasis on similarity)
    /// * `beta` - Weight for graph distance (higher = more emphasis on graph proximity)
    pub fn new(alpha: f32, beta: f32) -> Self {
        Self { alpha, beta }
    }
}

/// Result of a hybrid query including both vector and graph metrics.
#[derive(Debug, Clone)]
pub struct HybridResult {
    /// Node ID of this result.
    pub id: NodeId,
    /// Combined hybrid score (higher is better).
    pub score: f32,
    /// L2 distance from query vector.
    pub vector_distance: f32,
    /// Number of hops from start node.
    pub graph_distance: usize,
    /// BFS path from start node to this node.
    pub path: Vec<NodeId>,
}

impl HybridResult {
    /// Creates a new hybrid result.
    pub fn new(
        id: NodeId,
        score: f32,
        vector_distance: f32,
        graph_distance: usize,
        path: Vec<NodeId>,
    ) -> Self {
        Self {
            id,
            score,
            vector_distance,
            graph_distance,
            path,
        }
    }
}

/// Computes the hybrid score combining vector similarity and graph distance.
///
/// The score is computed as:
/// `score = alpha * (1 - normalized_vector_distance) + beta * (1 / (1 + graph_distance))`
///
/// This means:
/// - Higher alpha = more weight on vector similarity
/// - Higher beta = more weight on graph proximity
/// - Closer vectors and shorter graph paths result in higher scores
///
/// # Arguments
///
/// * `vec_dist` - L2 distance from query vector (lower is better)
/// * `graph_dist` - Number of hops from start node (lower is better)
/// * `params` - Hybrid scoring parameters
///
/// # Returns
///
/// A score where higher values indicate better matches.
pub fn compute_hybrid_score(vec_dist: f32, graph_dist: usize, params: &HybridParams) -> f32 {
    // Normalize vector distance to similarity (0-1 range, clamped)
    // Using min(1.0, vec_dist) to cap at 1.0 for normalization
    let vec_sim = 1.0 - vec_dist.min(1.0);

    // Convert graph distance to similarity (decreases with distance)
    let graph_sim = 1.0 / (1.0 + graph_dist as f32);

    params.alpha * vec_sim + params.beta * graph_sim
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = HybridParams::default();
        assert!((params.alpha - 0.5).abs() < 1e-6);
        assert!((params.beta - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_hybrid_score_identical() {
        let params = HybridParams::new(0.5, 0.5);
        // Perfect match: vector distance 0, graph distance 0
        let score = compute_hybrid_score(0.0, 0, &params);
        // vec_sim = 1.0, graph_sim = 1.0
        // score = 0.5 * 1.0 + 0.5 * 1.0 = 1.0
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_hybrid_score_far_vector() {
        let params = HybridParams::new(0.5, 0.5);
        // Far vector (distance >= 1.0), close graph
        let score = compute_hybrid_score(1.0, 0, &params);
        // vec_sim = 0.0, graph_sim = 1.0
        // score = 0.5 * 0.0 + 0.5 * 1.0 = 0.5
        assert!((score - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_hybrid_score_far_graph() {
        let params = HybridParams::new(0.5, 0.5);
        // Close vector, far graph
        let score = compute_hybrid_score(0.0, 9, &params);
        // vec_sim = 1.0, graph_sim = 1/10 = 0.1
        // score = 0.5 * 1.0 + 0.5 * 0.1 = 0.55
        assert!((score - 0.55).abs() < 1e-6);
    }

    #[test]
    fn test_hybrid_score_alpha_only() {
        let params = HybridParams::new(1.0, 0.0);
        // Only vector matters
        let score = compute_hybrid_score(0.5, 100, &params);
        // vec_sim = 0.5, graph ignored
        assert!((score - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_hybrid_score_beta_only() {
        let params = HybridParams::new(0.0, 1.0);
        // Only graph matters
        let score = compute_hybrid_score(10.0, 1, &params);
        // graph_sim = 1/2 = 0.5, vector ignored
        assert!((score - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_hybrid_score_capped_vector() {
        let params = HybridParams::new(1.0, 0.0);
        // Vector distance > 1.0 should be capped
        let score = compute_hybrid_score(5.0, 0, &params);
        // vec_sim = 1.0 - 1.0 = 0.0 (capped at 1.0)
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_hybrid_result_creation() {
        let result = HybridResult::new(42, 0.85, 0.15, 2, vec![1, 5, 42]);
        assert_eq!(result.id, 42);
        assert!((result.score - 0.85).abs() < 1e-6);
        assert!((result.vector_distance - 0.15).abs() < 1e-6);
        assert_eq!(result.graph_distance, 2);
        assert_eq!(result.path, vec![1, 5, 42]);
    }
}
