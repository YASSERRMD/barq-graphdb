//! Graph index for efficient traversal operations.
//!
//! This module provides a graph index structure using adjacency lists
//! for fast neighbor lookups and BFS traversal.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::NodeId;

/// In-memory graph index backed by adjacency lists.
///
/// Provides O(1) neighbor lookups and efficient BFS traversal
/// for exploring connected nodes.
#[derive(Debug, Default)]
pub struct GraphIndex {
    /// Adjacency list mapping each node to its outgoing neighbors.
    adjacency: HashMap<NodeId, Vec<NodeId>>,
}

impl GraphIndex {
    /// Creates a new empty graph index.
    ///
    /// # Returns
    ///
    /// A new `GraphIndex` with no edges.
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    /// Adds a directed edge from one node to another.
    ///
    /// If the source node doesn't exist in the index, it will be created.
    /// Duplicate edges are allowed.
    ///
    /// # Arguments
    ///
    /// * `from` - Source node ID
    /// * `to` - Target node ID
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.adjacency.entry(from).or_default().push(to);
        // Ensure target node exists in the adjacency map (may have no outgoing edges)
        self.adjacency.entry(to).or_default();
    }

    /// Returns the neighbors (outgoing edges) of a node.
    ///
    /// # Arguments
    ///
    /// * `id` - Node ID to look up
    ///
    /// # Returns
    ///
    /// An `Option` containing a slice of neighbor IDs, or `None` if
    /// the node doesn't exist in the index.
    pub fn neighbors(&self, id: NodeId) -> Option<&[NodeId]> {
        self.adjacency.get(&id).map(|v| v.as_slice())
    }

    /// Performs BFS traversal from a start node up to a maximum depth.
    ///
    /// Returns all nodes reachable within `max_hops` edges from the start.
    /// The start node is included in the result if it exists in the graph.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting node ID for BFS
    /// * `max_hops` - Maximum number of edges to traverse (depth limit)
    ///
    /// # Returns
    ///
    /// A vector of node IDs visited during BFS, in order of discovery.
    pub fn bfs_hops(&self, start: NodeId, max_hops: usize) -> Vec<NodeId> {
        // Return empty if start node doesn't exist
        if !self.adjacency.contains_key(&start) {
            return Vec::new();
        }

        let mut visited = HashSet::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Queue entries are (node_id, current_depth)
        queue.push_back((start, 0));
        visited.insert(start);
        result.push(start);

        while let Some((current, depth)) = queue.pop_front() {
            // Stop exploring further if we've reached max depth
            if depth >= max_hops {
                continue;
            }

            // Explore neighbors
            if let Some(neighbors) = self.adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        result.push(neighbor);
                        queue.push_back((neighbor, depth + 1));
                    }
                }
            }
        }

        result
    }

    /// Returns the number of nodes in the graph index.
    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Returns the total number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|v| v.len()).sum()
    }

    /// Checks if a node exists in the graph index.
    pub fn contains_node(&self, id: NodeId) -> bool {
        self.adjacency.contains_key(&id)
    }

    /// Clears all nodes and edges from the graph index.
    pub fn clear(&mut self) {
        self.adjacency.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph_is_empty() {
        let graph = GraphIndex::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_edge_creates_nodes() {
        let mut graph = GraphIndex::new();
        graph.add_edge(1, 2);

        assert!(graph.contains_node(1));
        assert!(graph.contains_node(2));
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_neighbors() {
        let mut graph = GraphIndex::new();
        graph.add_edge(1, 2);
        graph.add_edge(1, 3);
        graph.add_edge(1, 4);

        let neighbors = graph.neighbors(1).unwrap();
        assert_eq!(neighbors, &[2, 3, 4]);

        // Node with no outgoing edges
        let neighbors2 = graph.neighbors(2).unwrap();
        assert!(neighbors2.is_empty());

        // Non-existent node
        assert!(graph.neighbors(999).is_none());
    }

    #[test]
    fn test_bfs_simple_chain() {
        // 1 -> 2 -> 3 -> 4 -> 5
        let mut graph = GraphIndex::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 4);
        graph.add_edge(4, 5);

        // 0 hops: just start node
        let result = graph.bfs_hops(1, 0);
        assert_eq!(result, vec![1]);

        // 1 hop: 1 and 2
        let result = graph.bfs_hops(1, 1);
        assert_eq!(result, vec![1, 2]);

        // 2 hops: 1, 2, 3
        let result = graph.bfs_hops(1, 2);
        assert_eq!(result, vec![1, 2, 3]);

        // All hops
        let result = graph.bfs_hops(1, 10);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_bfs_tree_structure() {
        //     1
        //    / \
        //   2   3
        //  / \
        // 4   5
        let mut graph = GraphIndex::new();
        graph.add_edge(1, 2);
        graph.add_edge(1, 3);
        graph.add_edge(2, 4);
        graph.add_edge(2, 5);

        let result = graph.bfs_hops(1, 1);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));

        let result = graph.bfs_hops(1, 2);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_bfs_with_cycle() {
        // 1 -> 2 -> 3 -> 1 (cycle)
        let mut graph = GraphIndex::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 1);

        // Should not infinite loop
        let result = graph.bfs_hops(1, 10);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
    }

    #[test]
    fn test_bfs_nonexistent_start() {
        let graph = GraphIndex::new();
        let result = graph.bfs_hops(999, 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_bfs_isolated_node() {
        let mut graph = GraphIndex::new();
        graph.add_edge(1, 1); // Self-loop, but still creates node 1

        // Reset for isolated node test
        let mut graph2 = GraphIndex::new();
        graph2.adjacency.insert(1, Vec::new());

        let result = graph2.bfs_hops(1, 5);
        assert_eq!(result, vec![1]);
    }
}
