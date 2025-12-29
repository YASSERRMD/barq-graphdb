//! Storage engine for Barq-GraphDB.
//!
//! This module provides the core storage functionality including:
//! - Append-only Write-Ahead Log (WAL) for durability
//! - In-memory HashMap for fast node lookups
//! - Persistence and recovery from disk

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{Node, NodeId};

/// Configuration options for opening a database.
#[derive(Debug, Clone)]
pub struct DbOptions {
    /// Path to the database directory.
    pub path: PathBuf,
}

impl DbOptions {
    /// Creates new database options with the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory where database files will be stored
    ///
    /// # Returns
    ///
    /// A new `DbOptions` instance.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

/// WAL record kinds for different operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
enum WalRecord {
    /// A node was added or updated.
    #[serde(rename = "node")]
    Node(Node),
}

/// The main database struct providing storage operations.
///
/// `BarqGraphDb` manages an append-only WAL for durability and
/// an in-memory HashMap for fast node lookups.
pub struct BarqGraphDb {
    /// Database configuration options.
    options: DbOptions,
    /// File handle for the WAL.
    wal: File,
    /// In-memory node storage indexed by NodeId.
    nodes: HashMap<NodeId, Node>,
}

impl BarqGraphDb {
    /// Opens or creates a database at the specified path.
    ///
    /// If the database directory doesn't exist, it will be created.
    /// Existing WAL records will be replayed to restore state.
    ///
    /// # Arguments
    ///
    /// * `opts` - Database configuration options
    ///
    /// # Returns
    ///
    /// A `Result` containing the opened database or an error.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The directory cannot be created
    /// - The WAL file cannot be opened
    /// - Existing WAL records are corrupted
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use barq_graphdb::storage::{BarqGraphDb, DbOptions};
    /// use std::path::PathBuf;
    ///
    /// let opts = DbOptions::new(PathBuf::from("./my_db"));
    /// let db = BarqGraphDb::open(opts).unwrap();
    /// ```
    pub fn open(opts: DbOptions) -> Result<Self> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&opts.path)
            .with_context(|| format!("Failed to create database directory: {:?}", opts.path))?;

        let wal_path = opts.path.join("wal.log");

        // Load existing records if WAL exists
        let nodes = if wal_path.exists() {
            Self::load_wal(&wal_path).with_context(|| "Failed to load WAL")?
        } else {
            HashMap::new()
        };

        // Open WAL file for appending
        let wal = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&wal_path)
            .with_context(|| format!("Failed to open WAL file: {:?}", wal_path))?;

        Ok(Self {
            options: opts,
            wal,
            nodes,
        })
    }

    /// Loads WAL records from disk and reconstructs the node map.
    ///
    /// # Arguments
    ///
    /// * `wal_path` - Path to the WAL file
    ///
    /// # Returns
    ///
    /// A HashMap of nodes loaded from the WAL.
    fn load_wal(wal_path: &PathBuf) -> Result<HashMap<NodeId, Node>> {
        let file = File::open(wal_path)
            .with_context(|| format!("Failed to open WAL for reading: {:?}", wal_path))?;

        let reader = BufReader::new(file);
        let mut nodes = HashMap::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line =
                line_result.with_context(|| format!("Failed to read WAL line {}", line_num + 1))?;

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            let record: WalRecord = serde_json::from_str(&line)
                .with_context(|| format!("Failed to parse WAL record at line {}", line_num + 1))?;

            match record {
                WalRecord::Node(node) => {
                    nodes.insert(node.id, node);
                }
            }
        }

        Ok(nodes)
    }

    /// Appends a node to the database.
    ///
    /// The node is written to the WAL for durability and added to the
    /// in-memory index for fast lookups.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to append
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Serialization fails
    /// - Writing to the WAL fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use barq_graphdb::storage::{BarqGraphDb, DbOptions};
    /// use barq_graphdb::Node;
    /// use std::path::PathBuf;
    ///
    /// let opts = DbOptions::new(PathBuf::from("./my_db"));
    /// let mut db = BarqGraphDb::open(opts).unwrap();
    ///
    /// let node = Node::new(1, "example".to_string());
    /// db.append_node(node).unwrap();
    /// ```
    pub fn append_node(&mut self, node: Node) -> Result<()> {
        let record = WalRecord::Node(node.clone());

        // Serialize to JSON
        let json =
            serde_json::to_string(&record).with_context(|| "Failed to serialize node to JSON")?;

        // Append to WAL with newline
        writeln!(self.wal, "{}", json).with_context(|| "Failed to write node to WAL")?;

        // Flush to ensure durability
        self.wal.flush().with_context(|| "Failed to flush WAL")?;

        // Update in-memory index
        self.nodes.insert(node.id, node);

        Ok(())
    }

    /// Returns a reference to the in-memory node map.
    ///
    /// This is primarily used for testing and debugging.
    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }

    /// Gets a node by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID to look up
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the node if found.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Returns the number of nodes in the database.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the database path.
    pub fn path(&self) -> &PathBuf {
        &self.options.path
    }

    /// Lists all nodes in the database.
    ///
    /// # Returns
    ///
    /// A vector of references to all nodes.
    pub fn list_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_new_database() {
        let dir = TempDir::new().unwrap();
        let opts = DbOptions::new(dir.path().to_path_buf());

        let db = BarqGraphDb::open(opts).unwrap();
        assert_eq!(db.node_count(), 0);
    }

    #[test]
    fn test_append_and_retrieve_node() {
        let dir = TempDir::new().unwrap();
        let opts = DbOptions::new(dir.path().to_path_buf());

        let mut db = BarqGraphDb::open(opts).unwrap();

        let node = Node::new(1, "test_node".to_string());
        db.append_node(node.clone()).unwrap();

        assert_eq!(db.node_count(), 1);
        let retrieved = db.get_node(1).unwrap();
        assert_eq!(retrieved.id, 1);
        assert_eq!(retrieved.label, "test_node");
    }

    #[test]
    fn test_append_and_persist() {
        let dir = TempDir::new().unwrap();
        let opts = DbOptions::new(dir.path().to_path_buf());

        // Open, add nodes, close
        {
            let mut db = BarqGraphDb::open(opts.clone()).unwrap();
            let node = Node {
                id: 1,
                label: "test".to_string(),
                embedding: vec![],
                edges: vec![],
                timestamp: 0,
                agent_id: None,
                rule_tags: vec![],
            };
            db.append_node(node).unwrap();
        }

        // Reopen and verify
        let db2 = BarqGraphDb::open(opts).unwrap();
        assert!(db2.nodes().contains_key(&1));
        assert_eq!(db2.get_node(1).unwrap().label, "test");
    }

    #[test]
    fn test_multiple_nodes() {
        let dir = TempDir::new().unwrap();
        let opts = DbOptions::new(dir.path().to_path_buf());

        let mut db = BarqGraphDb::open(opts.clone()).unwrap();

        for i in 1..=10 {
            let node = Node::new(i, format!("node_{}", i));
            db.append_node(node).unwrap();
        }

        assert_eq!(db.node_count(), 10);

        // Reopen and verify all nodes persist
        drop(db);
        let db2 = BarqGraphDb::open(opts).unwrap();
        assert_eq!(db2.node_count(), 10);

        for i in 1..=10 {
            let node = db2.get_node(i).unwrap();
            assert_eq!(node.label, format!("node_{}", i));
        }
    }

    #[test]
    fn test_node_update_in_wal() {
        let dir = TempDir::new().unwrap();
        let opts = DbOptions::new(dir.path().to_path_buf());

        // Add node, then update it
        {
            let mut db = BarqGraphDb::open(opts.clone()).unwrap();
            let node = Node::new(1, "original".to_string());
            db.append_node(node).unwrap();

            // Update by appending again with same ID
            let updated_node = Node::new(1, "updated".to_string());
            db.append_node(updated_node).unwrap();
        }

        // Reopen and verify update persisted
        let db2 = BarqGraphDb::open(opts).unwrap();
        assert_eq!(db2.node_count(), 1);
        assert_eq!(db2.get_node(1).unwrap().label, "updated");
    }
}
