//! Integration tests for Phase 2: Vector Index & kNN Search.
//!
//! These tests verify vector embedding operations including
//! set_embedding, knn_search, and persistence.

use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use barq_graphdb::Node;
use tempfile::TempDir;

/// Tests the complete Phase 2 workflow:
/// 1. Add nodes with embeddings
/// 2. Set additional embeddings
/// 3. Query for kNN
/// 4. Verify distances are correct
/// 5. Verify persistence
#[test]
fn test_phase2_workflow() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();

        // Add nodes
        for i in 1..=5 {
            let node = Node::new(i, format!("node_{}", i));
            db.append_node(node).unwrap();
        }

        // Set embeddings in 2D space
        db.set_embedding(1, vec![0.0, 0.0]).unwrap();
        db.set_embedding(2, vec![1.0, 0.0]).unwrap();
        db.set_embedding(3, vec![0.0, 1.0]).unwrap();
        db.set_embedding(4, vec![1.0, 1.0]).unwrap();
        db.set_embedding(5, vec![5.0, 5.0]).unwrap();

        assert_eq!(db.vector_count(), 5);

        // Query from origin - node 1 should be closest
        let results = db.knn_search(&[0.0, 0.0], 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 1); // Closest is node 1
        assert!((results[0].1 - 0.0).abs() < 1e-6); // Distance 0

        // Verify distances are sorted
        for i in 0..results.len() - 1 {
            assert!(results[i].1 <= results[i + 1].1);
        }

        // Query from far point - node 5 should be closest
        let results = db.knn_search(&[5.0, 5.0], 1);
        assert_eq!(results[0].0, 5);
    }

    // Test persistence
    {
        let db = BarqGraphDb::open(opts).unwrap();

        // Embeddings should persist
        assert_eq!(db.vector_count(), 5);

        // kNN should still work
        let results = db.knn_search(&[0.0, 0.0], 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 1);
    }
}

/// Tests kNN with various k values.
#[test]
fn test_knn_various_k() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Add nodes with 1D embeddings
    for i in 1..=10 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
        db.set_embedding(i, vec![i as f32]).unwrap();
    }

    // k=1: Should return closest
    let results = db.knn_search(&[5.5], 1);
    assert_eq!(results.len(), 1);
    // Either 5 or 6 is closest to 5.5
    assert!(results[0].0 == 5 || results[0].0 == 6);

    // k=5: Should return 5 results
    let results = db.knn_search(&[5.0], 5);
    assert_eq!(results.len(), 5);
    assert_eq!(results[0].0, 5); // Exact match

    // k larger than dataset
    let results = db.knn_search(&[0.0], 100);
    assert_eq!(results.len(), 10); // Only 10 nodes exist
}

/// Tests distance calculations.
#[test]
fn test_distance_accuracy() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Create nodes with known distances from origin
    db.append_node(Node::new(1, "origin".to_string())).unwrap();
    db.append_node(Node::new(2, "unit_x".to_string())).unwrap();
    db.append_node(Node::new(3, "unit_y".to_string())).unwrap();
    db.append_node(Node::new(4, "far".to_string())).unwrap();

    db.set_embedding(1, vec![0.0, 0.0]).unwrap();
    db.set_embedding(2, vec![1.0, 0.0]).unwrap();
    db.set_embedding(3, vec![0.0, 1.0]).unwrap();
    db.set_embedding(4, vec![3.0, 4.0]).unwrap(); // Distance 5 from origin

    let results = db.knn_search(&[0.0, 0.0], 4);

    // Check ordering and distances
    assert_eq!(results[0].0, 1);
    assert!((results[0].1 - 0.0).abs() < 1e-6);

    // Nodes 2 and 3 at distance 1
    let mid_nodes: Vec<_> = results[1..3].iter().map(|(id, _)| *id).collect();
    assert!(mid_nodes.contains(&2) || mid_nodes.contains(&3));
    assert!((results[1].1 - 1.0).abs() < 1e-6);
    assert!((results[2].1 - 1.0).abs() < 1e-6);

    // Node 4 at distance 5
    assert_eq!(results[3].0, 4);
    assert!((results[3].1 - 5.0).abs() < 1e-6);
}

/// Tests embedding updates.
#[test]
fn test_embedding_update() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();

        db.append_node(Node::new(1, "moving".to_string())).unwrap();
        db.set_embedding(1, vec![0.0, 0.0]).unwrap();

        // Query should find node 1 at origin
        let results = db.knn_search(&[0.0, 0.0], 1);
        assert!((results[0].1 - 0.0).abs() < 1e-6);

        // Update embedding
        db.set_embedding(1, vec![10.0, 10.0]).unwrap();

        // Now node 1 is far from origin
        let results = db.knn_search(&[0.0, 0.0], 1);
        assert!((results[0].1 - (10.0_f32.powi(2) * 2.0).sqrt()).abs() < 1e-5);
    }

    // Verify update persisted
    {
        let db = BarqGraphDb::open(opts).unwrap();
        let results = db.knn_search(&[10.0, 10.0], 1);
        assert_eq!(results[0].0, 1);
        assert!((results[0].1 - 0.0).abs() < 1e-6);
    }
}

/// Tests empty vector index.
#[test]
fn test_empty_vector_index() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let db = BarqGraphDb::open(opts).unwrap();

    assert_eq!(db.vector_count(), 0);
    let results = db.knn_search(&[0.0, 0.0], 5);
    assert!(results.is_empty());
}

/// Tests high-dimensional vectors.
#[test]
fn test_high_dimensional() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // 128-dimensional vectors (common embedding size)
    let dim = 128;
    for i in 1..=5 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
        let embedding: Vec<f32> = (0..dim).map(|j| (i * j) as f32 * 0.01).collect();
        db.set_embedding(i, embedding).unwrap();
    }

    assert_eq!(db.vector_count(), 5);

    // Query with matching dimension
    let query: Vec<f32> = (0..dim).map(|j| j as f32 * 0.01).collect();
    let results = db.knn_search(&query, 3);
    assert_eq!(results.len(), 3);

    // Node 1 should be closest (its pattern is 1*j*0.01)
    assert_eq!(results[0].0, 1);
}

/// Tests nodes with embeddings added during creation.
#[test]
fn test_node_with_initial_embedding() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();

        let mut node = Node::new(1, "with_embedding".to_string());
        node.embedding = vec![1.0, 2.0, 3.0];
        db.append_node(node).unwrap();

        // Embedding should be in the index
        assert_eq!(db.vector_count(), 1);

        let results = db.knn_search(&[1.0, 2.0, 3.0], 1);
        assert_eq!(results[0].0, 1);
        assert!((results[0].1 - 0.0).abs() < 1e-6);
    }

    // Verify persists
    {
        let db = BarqGraphDb::open(opts).unwrap();
        assert_eq!(db.vector_count(), 1);
    }
}
