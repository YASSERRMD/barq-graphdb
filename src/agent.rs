//! Agent decision tracking and audit trails.
//!
//! This module provides types and functionality for recording agent
//! decision traces, reasoning paths, and enabling audit trails.

use serde::{Deserialize, Serialize};

use crate::NodeId;

/// A record of an agent's decision, including the reasoning path.
///
/// Decision records capture the path an agent took through the graph,
/// the nodes it considered, and metadata about the decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionRecord {
    /// Unique identifier for this decision record.
    pub id: u64,
    /// ID of the agent that made this decision.
    pub agent_id: u64,
    /// Unix timestamp when this decision was recorded.
    pub created_at: u64,
    /// The starting node for this decision path.
    pub root_node: NodeId,
    /// Sequence of nodes visited during the decision.
    pub path: Vec<NodeId>,
    /// Confidence or quality score for this decision.
    pub score: f32,
    /// Optional human-readable notes about the decision.
    pub notes: Option<String>,
}

impl DecisionRecord {
    /// Creates a new decision record with the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this decision
    /// * `agent_id` - ID of the agent making the decision
    /// * `root_node` - Starting node for the decision path
    /// * `path` - Sequence of nodes in the decision path
    /// * `score` - Confidence score for the decision
    ///
    /// # Returns
    ///
    /// A new `DecisionRecord` with the current timestamp.
    pub fn new(id: u64, agent_id: u64, root_node: NodeId, path: Vec<NodeId>, score: f32) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            agent_id,
            created_at,
            root_node,
            path,
            score,
            notes: None,
        }
    }

    /// Creates a new decision record with a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this decision
    /// * `agent_id` - ID of the agent making the decision
    /// * `created_at` - Unix timestamp for the decision
    /// * `root_node` - Starting node for the decision path
    /// * `path` - Sequence of nodes in the decision path
    /// * `score` - Confidence score for the decision
    ///
    /// # Returns
    ///
    /// A new `DecisionRecord` with the specified timestamp.
    pub fn with_timestamp(
        id: u64,
        agent_id: u64,
        created_at: u64,
        root_node: NodeId,
        path: Vec<NodeId>,
        score: f32,
    ) -> Self {
        Self {
            id,
            agent_id,
            created_at,
            root_node,
            path,
            score,
            notes: None,
        }
    }

    /// Adds notes to the decision record.
    ///
    /// # Arguments
    ///
    /// * `notes` - Human-readable notes about the decision
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_record_creation() {
        let record = DecisionRecord::new(1, 42, 100, vec![100, 101, 102], 0.95);

        assert_eq!(record.id, 1);
        assert_eq!(record.agent_id, 42);
        assert_eq!(record.root_node, 100);
        assert_eq!(record.path, vec![100, 101, 102]);
        assert!((record.score - 0.95).abs() < 1e-6);
        assert!(record.notes.is_none());
        assert!(record.created_at > 0);
    }

    #[test]
    fn test_decision_record_with_timestamp() {
        let record = DecisionRecord::with_timestamp(1, 42, 1234567890, 100, vec![100, 101], 0.85);

        assert_eq!(record.created_at, 1234567890);
    }

    #[test]
    fn test_decision_record_with_notes() {
        let record = DecisionRecord::new(1, 42, 100, vec![100], 0.90)
            .with_notes("Important decision about vulnerability cascade".to_string());

        assert!(record.notes.is_some());
        assert!(record.notes.unwrap().contains("vulnerability"));
    }

    #[test]
    fn test_decision_record_serialization() {
        let record = DecisionRecord::with_timestamp(1, 42, 1000, 100, vec![100, 101], 0.75)
            .with_notes("Test note".to_string());

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: DecisionRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(record, deserialized);
    }
}
