//! Integration tests for Phase 4: Agent Metadata, Decision Graphs & Audit.
//!
//! These tests verify agent decision recording, querying, and persistence.

use barq_graphdb::agent::DecisionRecord;
use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use barq_graphdb::Node;
use tempfile::TempDir;

/// Tests the complete Phase 4 workflow:
/// 1. Create nodes
/// 2. Record decisions from multiple agents
/// 3. Query decisions by agent
/// 4. Verify persistence
#[test]
fn test_phase4_workflow() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    // Record decisions and verify
    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();

        // Add some nodes for context
        for i in 1..=10 {
            db.append_node(Node::new(i, format!("node_{}", i))).unwrap();
        }

        // Agent 1 decisions
        let decision1 = DecisionRecord::new(1, 1, 1, vec![1, 2, 3], 0.95)
            .with_notes("First decision by agent 1".to_string());
        db.record_decision(decision1).unwrap();

        let decision2 = DecisionRecord::new(2, 1, 5, vec![5, 6, 7], 0.85);
        db.record_decision(decision2).unwrap();

        // Agent 2 decisions
        let decision3 = DecisionRecord::new(3, 2, 1, vec![1, 4, 8], 0.90)
            .with_notes("Agent 2's analysis".to_string());
        db.record_decision(decision3).unwrap();

        assert_eq!(db.decision_count(), 3);

        // Query by agent
        let agent1_decisions = db.list_decisions_for_agent(1);
        assert_eq!(agent1_decisions.len(), 2);

        let agent2_decisions = db.list_decisions_for_agent(2);
        assert_eq!(agent2_decisions.len(), 1);

        // Query all
        let all_decisions = db.list_all_decisions();
        assert_eq!(all_decisions.len(), 3);
    }

    // Test persistence
    {
        let db = BarqGraphDb::open(opts).unwrap();

        assert_eq!(db.decision_count(), 3);

        let agent1_decisions = db.list_decisions_for_agent(1);
        assert_eq!(agent1_decisions.len(), 2);

        // Verify content
        let first = db.get_decision(1).unwrap();
        assert_eq!(first.agent_id, 1);
        assert_eq!(first.root_node, 1);
        assert_eq!(first.path, vec![1, 2, 3]);
        assert!((first.score - 0.95).abs() < 1e-6);
        assert!(first.notes.as_ref().unwrap().contains("First decision"));
    }
}

/// Tests decision record creation.
#[test]
fn test_decision_record_creation() {
    let record = DecisionRecord::new(1, 42, 100, vec![100, 101, 102], 0.95);

    assert_eq!(record.id, 1);
    assert_eq!(record.agent_id, 42);
    assert_eq!(record.root_node, 100);
    assert_eq!(record.path, vec![100, 101, 102]);
    assert!(record.created_at > 0);
    assert!(record.notes.is_none());
}

/// Tests decision with notes.
#[test]
fn test_decision_with_notes() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts.clone()).unwrap();

    let record = DecisionRecord::new(1, 1, 1, vec![1], 0.99)
        .with_notes("Vulnerability cascade detected".to_string());
    db.record_decision(record).unwrap();

    drop(db);
    let db2 = BarqGraphDb::open(opts).unwrap();

    let retrieved = db2.get_decision(1).unwrap();
    assert!(retrieved.notes.is_some());
    assert!(retrieved.notes.as_ref().unwrap().contains("cascade"));
}

/// Tests empty decision queries.
#[test]
fn test_no_decisions() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let db = BarqGraphDb::open(opts).unwrap();

    assert_eq!(db.decision_count(), 0);
    assert!(db.list_all_decisions().is_empty());
    assert!(db.list_decisions_for_agent(999).is_empty());
    assert!(db.get_decision(1).is_none());
}

/// Tests multiple agents with many decisions.
#[test]
fn test_multiple_agents() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts).unwrap();

    // 5 agents, 10 decisions each
    let mut decision_id = 1;
    for agent_id in 1..=5 {
        for _ in 0..10 {
            let record = DecisionRecord::new(
                decision_id,
                agent_id,
                1,
                vec![1, 2],
                0.5 + (decision_id as f32 * 0.01),
            );
            db.record_decision(record).unwrap();
            decision_id += 1;
        }
    }

    assert_eq!(db.decision_count(), 50);

    for agent_id in 1..=5 {
        let decisions = db.list_decisions_for_agent(agent_id);
        assert_eq!(decisions.len(), 10);
        for d in decisions {
            assert_eq!(d.agent_id, agent_id);
        }
    }
}

/// Tests decision with agent_id on nodes.
#[test]
fn test_agent_metadata_on_nodes() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());
    let mut db = BarqGraphDb::open(opts.clone()).unwrap();

    // Create node with agent metadata
    let mut node = Node::new(1, "agent_created".to_string());
    node.agent_id = Some(42);
    node.rule_tags = vec!["security".to_string(), "high".to_string()];
    db.append_node(node).unwrap();

    drop(db);
    let db2 = BarqGraphDb::open(opts).unwrap();

    let retrieved = db2.get_node(1).unwrap();
    assert_eq!(retrieved.agent_id, Some(42));
    assert_eq!(retrieved.rule_tags.len(), 2);
    assert!(retrieved.rule_tags.contains(&"security".to_string()));
}

/// Tests decision persistence across multiple sessions.
#[test]
fn test_decision_persistence_multiple_sessions() {
    let dir = TempDir::new().unwrap();
    let opts = DbOptions::new(dir.path().to_path_buf());

    // Session 1: Add some decisions
    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();
        db.record_decision(DecisionRecord::new(1, 1, 1, vec![1], 0.9))
            .unwrap();
        db.record_decision(DecisionRecord::new(2, 1, 2, vec![2], 0.8))
            .unwrap();
    }

    // Session 2: Add more decisions
    {
        let mut db = BarqGraphDb::open(opts.clone()).unwrap();
        assert_eq!(db.decision_count(), 2);
        db.record_decision(DecisionRecord::new(3, 2, 3, vec![3], 0.7))
            .unwrap();
    }

    // Session 3: Verify all persist
    {
        let db = BarqGraphDb::open(opts).unwrap();
        assert_eq!(db.decision_count(), 3);
        assert!(db.get_decision(1).is_some());
        assert!(db.get_decision(2).is_some());
        assert!(db.get_decision(3).is_some());
    }
}
