//! Integration tests for Phase 1: Graph Index & BFS.
//!
//! These tests verify the graph traversal functionality including
//! edge creation, neighbor lookups, and BFS traversal.

use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use barq_graphdb::Node;
use tempfile::TempDir;

/// Tests the complete Phase 1 workflow:
/// 1. Create nodes
/// 2. Add edges
/// 3. Query neighbors
/// 4. Run BFS traversal
/// 5. Verify persistence
#[test]
fn test_phase1_workflow() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    // Build a test graph:
    //     1 --> 2 --> 4
    //     |     |
    //     v     v
    //     3 --> 5 --> 6
    //           |
    //           v
    //           7
    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();

        // Add nodes
        for i in 1..=7 {
            let node = Node::new(i, format!("node_{}", i));
            db.append_node(node).unwrap();
        }

        // Add edges
        db.add_edge(1, 2, "CONNECTS").unwrap();
        db.add_edge(1, 3, "CONNECTS").unwrap();
        db.add_edge(2, 4, "CONNECTS").unwrap();
        db.add_edge(2, 5, "CONNECTS").unwrap();
        db.add_edge(3, 5, "CONNECTS").unwrap();
        db.add_edge(5, 6, "CONNECTS").unwrap();
        db.add_edge(5, 7, "CONNECTS").unwrap();

        // Verify edge count
        assert_eq!(db.edge_count(), 7);

        // Test neighbors
        let n1_neighbors = db.neighbors(1).unwrap();
        assert_eq!(n1_neighbors.len(), 2);
        assert!(n1_neighbors.contains(&2));
        assert!(n1_neighbors.contains(&3));

        let n5_neighbors = db.neighbors(5).unwrap();
        assert_eq!(n5_neighbors.len(), 2);
        assert!(n5_neighbors.contains(&6));
        assert!(n5_neighbors.contains(&7));

        // Test BFS from node 1
        let bfs_0 = db.bfs_hops(1, 0);
        assert_eq!(bfs_0, vec![1]);

        let bfs_1 = db.bfs_hops(1, 1);
        assert_eq!(bfs_1.len(), 3);
        assert!(bfs_1.contains(&1));
        assert!(bfs_1.contains(&2));
        assert!(bfs_1.contains(&3));

        let bfs_2 = db.bfs_hops(1, 2);
        assert_eq!(bfs_2.len(), 5);
        assert!(bfs_2.contains(&4));
        assert!(bfs_2.contains(&5));

        // Full traversal
        let bfs_all = db.bfs_hops(1, 10);
        assert_eq!(bfs_all.len(), 7);
    }

    // Test persistence - reopen and verify edges survived
    {
        let db = BarqGraphDb::open(opts).unwrap();

        // Edge count should be preserved
        assert_eq!(db.edge_count(), 7);

        // Neighbors should be preserved
        let n1_neighbors = db.neighbors(1).unwrap();
        assert_eq!(n1_neighbors.len(), 2);

        // BFS should work the same
        let bfs_all = db.bfs_hops(1, 10);
        assert_eq!(bfs_all.len(), 7);
    }
}

/// Tests BFS with a linear chain graph.
#[test]
fn test_bfs_linear_chain() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Create chain: 1 -> 2 -> 3 -> 4 -> 5
    for i in 1..=5 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
    }
    for i in 1..=4 {
        db.add_edge(i, i + 1, "NEXT").unwrap();
    }

    // Test various depths
    assert_eq!(db.bfs_hops(1, 0), vec![1]);
    assert_eq!(db.bfs_hops(1, 1), vec![1, 2]);
    assert_eq!(db.bfs_hops(1, 2), vec![1, 2, 3]);
    assert_eq!(db.bfs_hops(1, 3), vec![1, 2, 3, 4]);
    assert_eq!(db.bfs_hops(1, 4), vec![1, 2, 3, 4, 5]);
    assert_eq!(db.bfs_hops(1, 100), vec![1, 2, 3, 4, 5]);

    // Start from middle
    assert_eq!(db.bfs_hops(3, 2), vec![3, 4, 5]);
}

/// Tests BFS with cycles.
#[test]
fn test_bfs_with_cycles() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // Create cycle: 1 -> 2 -> 3 -> 1
    for i in 1..=3 {
        db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
    }
    db.add_edge(1, 2, "NEXT").unwrap();
    db.add_edge(2, 3, "NEXT").unwrap();
    db.add_edge(3, 1, "BACK").unwrap();

    // Should not infinite loop
    let result = db.bfs_hops(1, 100);
    assert_eq!(result.len(), 3);
    assert!(result.contains(&1));
    assert!(result.contains(&2));
    assert!(result.contains(&3));
}

/// Tests neighbors of nodes with no outgoing edges.
#[test]
fn test_neighbors_leaf_node() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    db.append_node(Node::new(1, "root".to_string())).unwrap();
    db.append_node(Node::new(2, "leaf".to_string())).unwrap();
    db.add_edge(1, 2, "CONNECTS").unwrap();

    // Node 2 has no outgoing edges
    let leaf_neighbors = db.neighbors(2).unwrap();
    assert!(leaf_neighbors.is_empty());

    // Non-existent node
    assert!(db.neighbors(999).is_none());
}

/// Tests edge persistence across restarts.
#[test]
fn test_edge_persistence() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    // Create and add edges
    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();
        db.append_node(Node::new(1, "a".to_string())).unwrap();
        db.append_node(Node::new(2, "b".to_string())).unwrap();
        db.append_node(Node::new(3, "c".to_string())).unwrap();

        db.add_edge(1, 2, "FIRST").unwrap();
        db.add_edge(2, 3, "SECOND").unwrap();
        db.add_edge(1, 3, "SHORTCUT").unwrap();

        assert_eq!(db.edge_count(), 3);
    }

    // Reopen and verify
    {
        let db = BarqGraphDb::open(opts).unwrap();
        assert_eq!(db.edge_count(), 3);

        let n1 = db.neighbors(1).unwrap();
        assert!(n1.contains(&2));
        assert!(n1.contains(&3));

        let n2 = db.neighbors(2).unwrap();
        assert!(n2.contains(&3));
    }
}

/// Tests adding edges without pre-existing nodes.
#[test]
fn test_edge_before_node() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts.clone()).unwrap();

    // Add edge before nodes exist
    db.add_edge(10, 20, "EARLY").unwrap();

    // Neighbors should work via adjacency
    let n10 = db.neighbors(10).unwrap();
    assert!(n10.contains(&20));

    // BFS should still work
    let bfs = db.bfs_hops(10, 1);
    assert!(bfs.contains(&10));
    assert!(bfs.contains(&20));

    // Persist and reload
    drop(db);
    let db2 = BarqGraphDb::open(opts).unwrap();
    assert_eq!(db2.edge_count(), 1);
}
