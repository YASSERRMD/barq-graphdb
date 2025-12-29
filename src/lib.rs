//! # Barq-GraphDB
//!
//! A production-grade graph+vector database designed for AI agents.
//!
//! Barq-GraphDB provides:
//! - Append-only WAL (Write-Ahead Log) for durability
//! - In-memory graph storage with adjacency lists
//! - Vector embeddings with kNN search
//! - Hybrid queries combining graph traversal and vector similarity
//! - Agent decision tracking and audit trails
//!
//! ## Example
//!
//! ```rust,no_run
//! use barq_graphdb::storage::{BarqGraphDb, DbOptions};
//! use barq_graphdb::Node;
//! use std::path::PathBuf;
//!
//! let opts = DbOptions::new(PathBuf::from("./my_db"));
//! let mut db = BarqGraphDb::open(opts).unwrap();
//! ```

pub mod error;
pub mod graph;
pub mod hybrid;
pub mod storage;
pub mod vector;

use serde::{Deserialize, Serialize};

/// Unique identifier for nodes in the graph.
pub type NodeId = u64;

/// Represents a directed edge between two nodes in the graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    /// Source node ID.
    pub from: NodeId,
    /// Target node ID.
    pub to: NodeId,
    /// Type/label of the edge (e.g., "CALLS", "DEPENDS_ON").
    pub edge_type: String,
}

/// Represents a node in the graph with optional vector embedding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// Human-readable label for the node.
    pub label: String,
    /// Vector embedding for similarity search.
    pub embedding: Vec<f32>,
    /// Outgoing edges from this node.
    pub edges: Vec<Edge>,
    /// Unix timestamp when this node was created.
    pub timestamp: u64,
    /// Optional agent ID that created this node.
    pub agent_id: Option<u64>,
    /// Tags for rule-based filtering and categorization.
    pub rule_tags: Vec<String>,
}

impl Node {
    /// Creates a new node with the given ID and label.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the node
    /// * `label` - Human-readable label
    ///
    /// # Returns
    ///
    /// A new `Node` with default values for other fields.
    pub fn new(id: NodeId, label: String) -> Self {
        Self {
            id,
            label,
            embedding: Vec::new(),
            edges: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            agent_id: None,
            rule_tags: Vec::new(),
        }
    }

    /// Creates a new node with a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the node
    /// * `label` - Human-readable label
    /// * `timestamp` - Unix timestamp
    ///
    /// # Returns
    ///
    /// A new `Node` with the specified timestamp.
    pub fn with_timestamp(id: NodeId, label: String, timestamp: u64) -> Self {
        Self {
            id,
            label,
            embedding: Vec::new(),
            edges: Vec::new(),
            timestamp,
            agent_id: None,
            rule_tags: Vec::new(),
        }
    }
}
