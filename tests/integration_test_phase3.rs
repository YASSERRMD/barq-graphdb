//! Integration tests for Phase 3: Hybrid Query (Vector + Graph).
//!
//! These tests verify hybrid query functionality combining vector
//! similarity with graph traversal distance.

use barq_graphdb::hybrid::HybridParams;
use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use barq_graphdb::Node;
use tempfile::TempDir;

/// Tests the complete Phase 3 workflow:
/// 1. Create a graph with nodes and edges
/// 2. Set embeddings for nodes
/// 3. Run hybrid queries with different parameters
/// 4. Verify scoring and ranking
#[test]
fn test_phase3_workflow() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Create a graph:
    //     1 --> 2 --> 3
    //     |
    //     v
    //     4 --> 5
    for i in 1..=5 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
    }
    db.add_edge(1, 2, "CONNECTS").unwrap();
    db.add_edge(2, 3, "CONNECTS").unwrap();
    db.add_edge(1, 4, "CONNECTS").unwrap();
    db.add_edge(4, 5, "CONNECTS").unwrap();

    // Set embeddings in 2D space
    // Query will be at [0, 0]
    db.set_embedding(1, vec![0.0, 0.0]).unwrap(); // At origin
    db.set_embedding(2, vec![1.0, 0.0]).unwrap(); // Distance 1
    db.set_embedding(3, vec![2.0, 0.0]).unwrap(); // Distance 2
    db.set_embedding(4, vec![0.0, 0.5]).unwrap(); // Distance 0.5 (closer vector)
    db.set_embedding(5, vec![0.0, 1.0]).unwrap(); // Distance 1

    // Hybrid query with equal weights
    let params = HybridParams::new(0.5, 0.5);
    let results = db.hybrid_query(&[0.0, 0.0], 1, 2, 5, params);

    // Should get nodes within 2 hops: 1, 2, 3, 4, 5
    assert_eq!(results.len(), 5);

    // Node 1 should be first (distance 0 to both vector and graph)
    assert_eq!(results[0].id, 1);
    assert!((results[0].score - 1.0).abs() < 1e-5);

    // Verify paths are included
    assert_eq!(results[0].path, vec![1]);
    for result in &results {
        assert!(!result.path.is_empty());
        assert_eq!(*result.path.first().unwrap(), 1); // All paths start from 1
    }
}

/// Tests alpha-only scoring (vector similarity only).
#[test]
fn test_hybrid_alpha_only() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Linear graph: 1 -> 2 -> 3
    for i in 1..=3 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
    }
    db.add_edge(1, 2, "NEXT").unwrap();
    db.add_edge(2, 3, "NEXT").unwrap();

    // Node 3 is closest to query, but farthest in graph
    db.set_embedding(1, vec![1.0]).unwrap();
    db.set_embedding(2, vec![0.5]).unwrap();
    db.set_embedding(3, vec![0.0]).unwrap(); // Closest to query [0.0]

    // Alpha=1.0: Only vector distance matters
    let params = HybridParams::new(1.0, 0.0);
    let results = db.hybrid_query(&[0.0], 1, 10, 3, params);

    // Node 3 should be first (closest vector)
    assert_eq!(results[0].id, 3);
    // Node 2 second
    assert_eq!(results[1].id, 2);
    // Node 1 last
    assert_eq!(results[2].id, 1);
}

/// Tests beta-only scoring (graph distance only).
#[test]
fn test_hybrid_beta_only() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Linear graph: 1 -> 2 -> 3
    for i in 1..=3 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
    }
    db.add_edge(1, 2, "NEXT").unwrap();
    db.add_edge(2, 3, "NEXT").unwrap();

    // Node 3 is closest to query, but farthest in graph
    db.set_embedding(1, vec![1.0]).unwrap(); // Farthest from query
    db.set_embedding(2, vec![0.5]).unwrap();
    db.set_embedding(3, vec![0.0]).unwrap(); // Closest to query

    // Beta=1.0: Only graph distance matters
    let params = HybridParams::new(0.0, 1.0);
    let results = db.hybrid_query(&[0.0], 1, 10, 3, params);

    // Node 1 should be first (graph distance 0)
    assert_eq!(results[0].id, 1);
    // Node 2 second (graph distance 1)
    assert_eq!(results[1].id, 2);
    // Node 3 last (graph distance 2)
    assert_eq!(results[2].id, 3);
}

/// Tests max_hops limiting.
#[test]
fn test_hybrid_max_hops() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Long chain: 1 -> 2 -> 3 -> 4 -> 5
    for i in 1..=5 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
        db.set_embedding(i, vec![i as f32]).unwrap();
    }
    for i in 1..=4 {
        db.add_edge(i, i + 1, "NEXT").unwrap();
    }

    let params = HybridParams::default();

    // max_hops=1: Only nodes 1, 2
    let results = db.hybrid_query(&[0.0], 1, 1, 10, params.clone());
    assert_eq!(results.len(), 2);

    // max_hops=2: Nodes 1, 2, 3
    let results = db.hybrid_query(&[0.0], 1, 2, 10, params.clone());
    assert_eq!(results.len(), 3);

    // max_hops=0: Only start node
    let results = db.hybrid_query(&[0.0], 1, 0, 10, params);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, 1);
}

/// Tests k limiting.
#[test]
fn test_hybrid_k_limit() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Create 10 connected nodes
    for i in 1..=10 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
        db.set_embedding(i, vec![i as f32]).unwrap();
        if i > 1 {
            db.add_edge(1, i, "CONNECTS").unwrap();
        }
    }

    let params = HybridParams::default();

    // k=3: Only top 3
    let results = db.hybrid_query(&[0.0], 1, 1, 3, params.clone());
    assert_eq!(results.len(), 3);

    // k=100: All 10
    let results = db.hybrid_query(&[0.0], 1, 1, 100, params);
    assert_eq!(results.len(), 10);
}

/// Tests empty results.
#[test]
fn test_hybrid_empty() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let db = BarqGraphDb::open(opts).unwrap();

    let params = HybridParams::default();

    // Non-existent start node
    let results = db.hybrid_query(&[0.0], 999, 10, 5, params);
    assert!(results.is_empty());
}

/// Tests nodes without embeddings are excluded.
#[test]
fn test_hybrid_missing_embeddings() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // 3 nodes, only 2 have embeddings
    for i in 1..=3 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
    }
    db.add_edge(1, 2, "NEXT").unwrap();
    db.add_edge(2, 3, "NEXT").unwrap();

    db.set_embedding(1, vec![0.0]).unwrap();
    // Node 2 has no embedding
    db.set_embedding(3, vec![2.0]).unwrap();

    let params = HybridParams::default();
    let results = db.hybrid_query(&[0.0], 1, 10, 10, params);

    // Only nodes 1 and 3 should be in results
    assert_eq!(results.len(), 2);
    let ids: Vec<_> = results.iter().map(|r| r.id).collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&3));
    assert!(!ids.contains(&2));
}

/// Tests path reconstruction.
#[test]
fn test_hybrid_paths() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Graph: 1 -> 2 -> 3 -> 4
    for i in 1..=4 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
        db.set_embedding(i, vec![0.0]).unwrap();
    }
    db.add_edge(1, 2, "NEXT").unwrap();
    db.add_edge(2, 3, "NEXT").unwrap();
    db.add_edge(3, 4, "NEXT").unwrap();

    let params = HybridParams::default();
    let results = db.hybrid_query(&[0.0], 1, 10, 10, params);

    // Find node 4's result
    let node4_result = results.iter().find(|r| r.id == 4).unwrap();
    assert_eq!(node4_result.path, vec![1, 2, 3, 4]);
    assert_eq!(node4_result.graph_distance, 3);

    // Node 1's path should be just itself
    let node1_result = results.iter().find(|r| r.id == 1).unwrap();
    assert_eq!(node1_result.path, vec![1]);
    assert_eq!(node1_result.graph_distance, 0);
}
