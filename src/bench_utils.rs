//! Benchmark utilities for data generation and test scenarios.
//!
//! This module provides utilities for generating test data for benchmarks,
//! including nodes, edges, and realistic scenarios.

use rand::Rng;

use crate::Node;

/// Generates N random nodes with optional embeddings.
///
/// # Arguments
///
/// * `count` - Number of nodes to generate
/// * `embedding_dim` - Dimension of embedding vectors (0 for no embeddings)
///
/// # Returns
///
/// A vector of randomly generated nodes.
pub fn generate_random_nodes(count: usize, embedding_dim: usize) -> Vec<Node> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|i| {
            let embedding = if embedding_dim > 0 {
                (0..embedding_dim)
                    .map(|_| rng.gen_range(0.0..1.0))
                    .collect()
            } else {
                vec![]
            };
            Node {
                id: i as u64,
                label: format!("node_{}", i),
                embedding,
                edges: vec![],
                timestamp: 0,
                agent_id: None,
                rule_tags: vec![],
            }
        })
        .collect()
}

/// Generates random edges following a scale-free distribution.
///
/// # Arguments
///
/// * `nodes` - Number of nodes in the graph
/// * `edges_per_node` - Average number of edges per node
///
/// # Returns
///
/// A vector of (from, to) edge tuples.
pub fn generate_scale_free_edges(nodes: usize, edges_per_node: usize) -> Vec<(u64, u64)> {
    let mut rng = rand::thread_rng();
    let mut result = vec![];

    for i in 0..nodes {
        for _ in 0..edges_per_node {
            let to = rng.gen_range(0..nodes);
            if i != to {
                result.push((i as u64, to as u64));
            }
        }
    }
    result
}

/// Generates a linear chain of nodes (for predictable BFS testing).
///
/// # Arguments
///
/// * `count` - Number of nodes in the chain
///
/// # Returns
///
/// A vector of (from, to) edge tuples forming a chain.
pub fn generate_linear_chain(count: usize) -> Vec<(u64, u64)> {
    (0..count.saturating_sub(1))
        .map(|i| (i as u64, (i + 1) as u64))
        .collect()
}

/// Generates a tree structure with specified branching factor.
///
/// # Arguments
///
/// * `depth` - Depth of the tree
/// * `branching_factor` - Number of children per node
///
/// # Returns
///
/// A vector of (from, to) edge tuples forming a tree.
pub fn generate_tree(depth: usize, branching_factor: usize) -> Vec<(u64, u64)> {
    let mut edges = vec![];
    let mut node_id = 0u64;
    let mut queue = vec![0u64];

    for _ in 0..depth {
        let mut next_queue = vec![];
        for parent in queue {
            for _ in 0..branching_factor {
                node_id += 1;
                edges.push((parent, node_id));
                next_queue.push(node_id);
            }
        }
        queue = next_queue;
    }
    edges
}

/// Generates a random query vector.
///
/// # Arguments
///
/// * `dim` - Dimension of the vector
///
/// # Returns
///
/// A random f32 vector.
pub fn generate_random_query(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(0.0..1.0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_nodes() {
        let nodes = generate_random_nodes(100, 128);
        assert_eq!(nodes.len(), 100);
        assert_eq!(nodes[0].embedding.len(), 128);
    }

    #[test]
    fn test_generate_scale_free_edges() {
        let edges = generate_scale_free_edges(100, 3);
        assert!(edges.len() > 200); // ~300 edges expected
    }

    #[test]
    fn test_generate_linear_chain() {
        let edges = generate_linear_chain(10);
        assert_eq!(edges.len(), 9);
        assert_eq!(edges[0], (0, 1));
        assert_eq!(edges[8], (8, 9));
    }

    #[test]
    fn test_generate_tree() {
        let edges = generate_tree(2, 2);
        // Depth 2, branching 2: 1 + 2 + 4 = 7 nodes, 6 edges
        assert_eq!(edges.len(), 6);
    }
}
