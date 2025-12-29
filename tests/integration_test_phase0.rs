//! Integration tests for Phase 0: Core Storage & Node Model.
//!
//! These tests verify the end-to-end functionality of the storage layer,
//! including persistence across database restarts.

use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use barq_graphdb::Node;
use tempfile::TempDir;

/// Tests the complete Phase 0 workflow:
/// 1. Create a new database
/// 2. Add multiple nodes
/// 3. Close and reopen the database
/// 4. Verify all nodes persist correctly
#[test]
fn test_phase0_workflow() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    // Step 1 & 2: Create DB and add nodes
    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();

        // Add first node
        let node1 = Node {
            id: 1,
            label: "function_main".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            edges: vec![],
            timestamp: 1000,
            agent_id: Some(42),
            rule_tags: vec!["entry_point".to_string()],
        };
        db.append_node(node1).unwrap();

        // Add second node
        let node2 = Node {
            id: 2,
            label: "function_helper".to_string(),
            embedding: vec![0.4, 0.5, 0.6],
            edges: vec![],
            timestamp: 1001,
            agent_id: Some(42),
            rule_tags: vec!["utility".to_string()],
        };
        db.append_node(node2).unwrap();

        // Add third node
        let node3 = Node {
            id: 3,
            label: "class_processor".to_string(),
            embedding: vec![0.7, 0.8, 0.9],
            edges: vec![],
            timestamp: 1002,
            agent_id: None,
            rule_tags: vec!["core".to_string(), "processing".to_string()],
        };
        db.append_node(node3).unwrap();

        // Verify in-memory state
        assert_eq!(db.node_count(), 3);
    }

    // Step 3 & 4: Reopen and verify persistence
    {
        let db = BarqGraphDb::open(opts).unwrap();

        // Verify node count
        assert_eq!(db.node_count(), 3, "Expected 3 nodes after reopening");

        // Verify node 1
        let node1 = db.get_node(1).expect("Node 1 should exist");
        assert_eq!(node1.label, "function_main");
        assert_eq!(node1.embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(node1.timestamp, 1000);
        assert_eq!(node1.agent_id, Some(42));
        assert_eq!(node1.rule_tags, vec!["entry_point".to_string()]);

        // Verify node 2
        let node2 = db.get_node(2).expect("Node 2 should exist");
        assert_eq!(node2.label, "function_helper");
        assert_eq!(node2.agent_id, Some(42));

        // Verify node 3
        let node3 = db.get_node(3).expect("Node 3 should exist");
        assert_eq!(node3.label, "class_processor");
        assert_eq!(node3.agent_id, None);
        assert_eq!(
            node3.rule_tags,
            vec!["core".to_string(), "processing".to_string()]
        );
    }
}

/// Tests that nodes with the same ID are properly updated (last-write-wins).
#[test]
fn test_node_update_persistence() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    // Add node, then update it
    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();

        let node_v1 = Node::new(100, "original_label".to_string());
        db.append_node(node_v1).unwrap();

        let mut node_v2 = Node::new(100, "updated_label".to_string());
        node_v2.agent_id = Some(999);
        db.append_node(node_v2).unwrap();
    }

    // Verify update persisted
    {
        let db = BarqGraphDb::open(opts).unwrap();
        assert_eq!(db.node_count(), 1);

        let node = db.get_node(100).unwrap();
        assert_eq!(node.label, "updated_label");
        assert_eq!(node.agent_id, Some(999));
    }
}

/// Tests empty database behavior.
#[test]
fn test_empty_database() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    // Create empty DB
    {
        let db = BarqGraphDb::open(opts.clone()).unwrap();
        assert_eq!(db.node_count(), 0);
        assert!(db.list_nodes().is_empty());
        assert!(db.get_node(1).is_none());
    }

    // Reopen empty DB
    {
        let db = BarqGraphDb::open(opts).unwrap();
        assert_eq!(db.node_count(), 0);
    }
}

/// Tests listing nodes returns all nodes.
#[test]
fn test_list_nodes() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    let mut db = BarqGraphDb::open(opts).unwrap();

    for i in 1..=5 {
        let node = Node::new(i, format!("node_{}", i));
        db.append_node(node).unwrap();
    }

    let nodes = db.list_nodes();
    assert_eq!(nodes.len(), 5);

    // Verify all IDs are present (order may vary)
    let ids: Vec<u64> = nodes.iter().map(|n| n.id).collect();
    for i in 1..=5 {
        assert!(ids.contains(&i), "Expected node {} to be in list", i);
    }
}
