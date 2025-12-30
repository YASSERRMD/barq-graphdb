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
use std::sync::Arc;
use std::time::Duration;

use crate::batch_indexer::BatchIndexer;
use crate::batch_queue::BatchQueue;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::agent::DecisionRecord;
use crate::vector::{HnswVectorIndex, LinearVectorIndex, VectorIndex};
use crate::{Edge, Node, NodeId};

/// Type alias for the node storage map.
type NodeMap = HashMap<NodeId, Node>;

/// Type alias for the adjacency list.
type AdjacencyMap = HashMap<NodeId, Vec<NodeId>>;

/// Type alias for vector storage during WAL load.
type VectorMap = HashMap<NodeId, Vec<f32>>;

/// Type alias for WAL load result.
type WalLoadResult = (NodeMap, AdjacencyMap, VectorMap, Vec<DecisionRecord>);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IndexType {
    Linear,
    Hnsw,
}

/// Configuration options for opening a database.
#[derive(Debug, Clone)]
pub struct DbOptions {
    /// Path to the database directory.
    pub path: PathBuf,
    /// Type of vector index to use.
    pub index_type: IndexType,
    /// Whether to flush WAL to disk after every write.
    pub sync_writes: bool,
    /// Whether to update vector index asynchronously.
    pub async_indexing: bool,
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
        Self {
            path,
            index_type: IndexType::Hnsw,
            sync_writes: true,
            async_indexing: false, // Default to synchronous for consistency
        }
    }
}

/// WAL record kinds for different operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum WalRecord {
    /// A node was added or updated.
    #[serde(rename = "node")]
    Node { data: Node },
    /// An edge was added between nodes.
    #[serde(rename = "edge")]
    Edge {
        from: NodeId,
        to: NodeId,
        edge_type: String,
    },
    /// An embedding was set for a node.
    #[serde(rename = "embedding")]
    Embedding { id: NodeId, vec: Vec<f32> },
    /// A decision record was added.
    #[serde(rename = "decision")]
    Decision { data: DecisionRecord },
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
    /// Adjacency list for graph traversal.
    adjacency: HashMap<NodeId, Vec<NodeId>>,
    /// Vector index for similarity search.
    vector_index: Arc<dyn VectorIndex>,
    /// Batch queue for async index updates.
    batch_queue: Option<BatchQueue>,
    /// Agent decision records.
    decisions: Vec<DecisionRecord>,
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
        let (nodes, adjacency, vectors, decisions) = if wal_path.exists() {
            Self::load_wal(&wal_path).with_context(|| "Failed to load WAL")?
        } else {
            (HashMap::new(), HashMap::new(), HashMap::new(), Vec::new())
        };

        // Build vector index based on configuration
        // Build vector index based on configuration
        let vector_index: Arc<dyn VectorIndex> = match opts.index_type {
            IndexType::Linear => Arc::new(LinearVectorIndex::new()),
            IndexType::Hnsw => Arc::new(HnswVectorIndex::new(1_000_000)),
        };
        for (id, embedding) in &vectors {
            vector_index.insert(*id, embedding);
        }
        // Also add embeddings from nodes
        for (id, node) in &nodes {
            if !node.embedding.is_empty() && !vector_index.contains(*id) {
                vector_index.insert(*id, &node.embedding);
            }
        }

        // Setup async thread if enabled
        let batch_queue = if opts.async_indexing {
            let queue = BatchQueue::new(100);
            BatchIndexer::start_background_thread(
                queue.clone(),
                vector_index.clone(),
                Duration::from_millis(10),
            );
            Some(queue)
        } else {
            None
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
            adjacency,
            vector_index,
            batch_queue,
            decisions,
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
    fn load_wal(wal_path: &PathBuf) -> Result<WalLoadResult> {
        let file = File::open(wal_path)
            .with_context(|| format!("Failed to open WAL for reading: {:?}", wal_path))?;

        let reader = BufReader::new(file);
        let mut nodes = HashMap::new();
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut vectors: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let mut decisions: Vec<DecisionRecord> = Vec::new();

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
                WalRecord::Node { data: node } => {
                    // Rebuild adjacency from node edges
                    for edge in &node.edges {
                        adjacency.entry(edge.from).or_default().push(edge.to);
                        adjacency.entry(edge.to).or_default();
                    }
                    // Store embedding if present
                    if !node.embedding.is_empty() {
                        vectors.insert(node.id, node.embedding.clone());
                    }
                    nodes.insert(node.id, node);
                }
                WalRecord::Edge { from, to, .. } => {
                    adjacency.entry(from).or_default().push(to);
                    adjacency.entry(to).or_default();
                }
                WalRecord::Embedding { id, vec } => {
                    vectors.insert(id, vec.clone());
                    // Update node embedding if node exists
                    if let Some(node) = nodes.get_mut(&id) {
                        node.embedding = vec;
                    }
                }
                WalRecord::Decision { data: decision } => {
                    decisions.push(decision);
                }
            }
        }

        Ok((nodes, adjacency, vectors, decisions))
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
        let record = WalRecord::Node { data: node.clone() };

        // Serialize to JSON
        let json =
            serde_json::to_string(&record).with_context(|| "Failed to serialize node to JSON")?;

        // Append to WAL with newline
        writeln!(self.wal, "{}", json).with_context(|| "Failed to write node to WAL")?;

        // Flush to ensure durability
        if self.options.sync_writes {
            self.wal.flush().with_context(|| "Failed to flush WAL")?;
        }

        // Rebuild adjacency from node edges
        for edge in &node.edges {
            self.adjacency.entry(edge.from).or_default().push(edge.to);
            self.adjacency.entry(edge.to).or_default();
        }

        // Add embedding to vector index if present
        if !node.embedding.is_empty() {
            if let Some(queue) = &self.batch_queue {
                queue.push(node.clone());
            } else {
                self.vector_index.insert(node.id, &node.embedding);
            }
        }

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

    /// Adds a directed edge between two nodes.
    ///
    /// The edge is written to the WAL for durability and the adjacency
    /// list is updated for fast neighbor lookups.
    ///
    /// # Arguments
    ///
    /// * `from` - Source node ID
    /// * `to` - Target node ID
    /// * `edge_type` - Type/label of the edge
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
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
    /// db.add_edge(1, 2, "CALLS").unwrap();
    /// ```
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, edge_type: &str) -> Result<()> {
        let record = WalRecord::Edge {
            from,
            to,
            edge_type: edge_type.to_string(),
        };

        // Serialize to JSON
        let json =
            serde_json::to_string(&record).with_context(|| "Failed to serialize edge to JSON")?;

        // Append to WAL
        writeln!(self.wal, "{}", json).with_context(|| "Failed to write edge to WAL")?;

        // Flush to ensure durability
        if self.options.sync_writes {
            self.wal.flush().with_context(|| "Failed to flush WAL")?;
        }

        // Update adjacency list
        self.adjacency.entry(from).or_default().push(to);
        self.adjacency.entry(to).or_default();

        // Also update the node's edges if the node exists
        if let Some(node) = self.nodes.get_mut(&from) {
            node.edges.push(Edge {
                from,
                to,
                edge_type: edge_type.to_string(),
            });
        }

        Ok(())
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
    /// the node doesn't exist in the adjacency list.
    pub fn neighbors(&self, id: NodeId) -> Option<&[NodeId]> {
        self.adjacency.get(&id).map(|v| v.as_slice())
    }

    /// Performs BFS traversal from a start node up to a maximum depth.
    ///
    /// Returns all nodes reachable within `max_hops` edges from the start.
    /// The start node is included in the result if it exists.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting node ID for BFS
    /// * `max_hops` - Maximum number of edges to traverse (depth limit)
    ///
    /// # Returns
    ///
    /// A vector of node IDs visited during BFS, in order of discovery.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use barq_graphdb::storage::{BarqGraphDb, DbOptions};
    /// use std::path::PathBuf;
    ///
    /// let opts = DbOptions::new(PathBuf::from("./my_db"));
    /// let db = BarqGraphDb::open(opts).unwrap();
    /// let reachable = db.bfs_hops(1, 2); // All nodes within 2 hops of node 1
    /// ```
    pub fn bfs_hops(&self, start: NodeId, max_hops: usize) -> Vec<NodeId> {
        use std::collections::{HashSet, VecDeque};

        // Check if start exists in nodes or adjacency
        if !self.nodes.contains_key(&start) && !self.adjacency.contains_key(&start) {
            return Vec::new();
        }

        let mut visited = HashSet::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Queue entries: (node_id, current_depth)
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

    /// Returns the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|v| v.len()).sum()
    }

    /// Sets the vector embedding for a node.
    ///
    /// The embedding is written to the WAL for durability and added
    /// to the vector index for similarity search.
    ///
    /// # Arguments
    ///
    /// * `id` - Node ID to set embedding for
    /// * `embedding` - Vector embedding to store
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use barq_graphdb::storage::{BarqGraphDb, DbOptions};
    /// use std::path::PathBuf;
    ///
    /// let opts = DbOptions::new(PathBuf::from("./my_db"));
    /// let mut db = BarqGraphDb::open(opts).unwrap();
    /// db.set_embedding(1, vec![0.1, 0.2, 0.3]).unwrap();
    /// ```
    pub fn set_embedding(&mut self, id: NodeId, embedding: Vec<f32>) -> Result<()> {
        let record = WalRecord::Embedding {
            id,
            vec: embedding.clone(),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&record)
            .with_context(|| "Failed to serialize embedding to JSON")?;

        // Append to WAL
        writeln!(self.wal, "{}", json).with_context(|| "Failed to write embedding to WAL")?;

        // Flush to ensure durability
        if self.options.sync_writes {
            self.wal.flush().with_context(|| "Failed to flush WAL")?;
        }

        // Update vector index
        // Update vector index
        if let Some(queue) = &self.batch_queue {
            let mut dummy_node = Node::new(id, String::new());
            dummy_node.embedding = embedding.clone();
            queue.push(dummy_node);
        } else {
            self.vector_index.insert(id, &embedding);
        }

        // Update node if it exists
        if let Some(node) = self.nodes.get_mut(&id) {
            node.embedding = embedding;
        }

        Ok(())
    }

    /// Finds the k nearest neighbors to a query vector.
    ///
    /// Uses L2 (Euclidean) distance for similarity comparison.
    ///
    /// # Arguments
    ///
    /// * `query` - Query vector for similarity search
    /// * `k` - Number of nearest neighbors to return
    ///
    /// # Returns
    ///
    /// A vector of (NodeId, distance) pairs sorted by distance ascending.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use barq_graphdb::storage::{BarqGraphDb, DbOptions};
    /// use std::path::PathBuf;
    ///
    /// let opts = DbOptions::new(PathBuf::from("./my_db"));
    /// let db = BarqGraphDb::open(opts).unwrap();
    /// let results = db.knn_search(&[0.1, 0.2, 0.3], 5);
    /// ```
    pub fn knn_search(&self, query: &[f32], k: usize) -> Vec<(NodeId, f32)> {
        self.vector_index.knn(query, k)
    }

    /// Returns the number of vectors in the index.
    pub fn vector_count(&self) -> usize {
        self.vector_index.len()
    }

    /// Gets the embedding for a node if it exists.
    pub fn get_embedding(&self, id: NodeId) -> Option<&[f32]> {
        self.nodes.get(&id).and_then(|n| {
            if n.embedding.is_empty() {
                None
            } else {
                Some(n.embedding.as_slice())
            }
        })
    }

    /// Performs a hybrid query combining vector similarity and graph distance.
    ///
    /// Starting from a given node, explores the graph via BFS up to max_hops,
    /// computes vector similarity for each visited node, and returns the top k
    /// results ranked by hybrid score.
    ///
    /// The hybrid score combines:
    /// - Vector similarity: `alpha * (1 - normalized_vector_distance)`
    /// - Graph proximity: `beta * (1 / (1 + graph_distance))`
    ///
    /// # Arguments
    ///
    /// * `query_embedding` - Query vector for similarity comparison
    /// * `start` - Starting node ID for BFS traversal
    /// * `max_hops` - Maximum BFS depth to explore
    /// * `k` - Number of top results to return
    /// * `params` - Hybrid scoring parameters (alpha, beta weights)
    ///
    /// # Returns
    ///
    /// A vector of `HybridResult` sorted by score descending.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use barq_graphdb::storage::{BarqGraphDb, DbOptions};
    /// use barq_graphdb::hybrid::HybridParams;
    /// use std::path::PathBuf;
    ///
    /// let opts = DbOptions::new(PathBuf::from("./my_db"));
    /// let db = BarqGraphDb::open(opts).unwrap();
    /// let params = HybridParams::new(0.7, 0.3);
    /// let results = db.hybrid_query(&[0.1, 0.2], 1, 3, 5, params);
    /// ```
    pub fn hybrid_query(
        &self,
        query_embedding: &[f32],
        start: NodeId,
        max_hops: usize,
        k: usize,
        params: crate::hybrid::HybridParams,
    ) -> Vec<crate::hybrid::HybridResult> {
        use crate::hybrid::{compute_hybrid_score, HybridResult};
        use crate::vector::l2_distance;
        use std::collections::{HashMap, HashSet, VecDeque};

        // Check if start exists
        if !self.nodes.contains_key(&start) && !self.adjacency.contains_key(&start) {
            return Vec::new();
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        // Track: (node_id, distance, path_to_node)
        let mut node_info: HashMap<NodeId, (usize, Vec<NodeId>)> = HashMap::new();

        // Initialize BFS from start
        queue.push_back((start, 0, vec![start]));
        visited.insert(start);
        node_info.insert(start, (0, vec![start]));

        while let Some((current, depth, path)) = queue.pop_front() {
            // Stop exploring further if we've reached max depth
            if depth >= max_hops {
                continue;
            }

            // Explore neighbors
            if let Some(neighbors) = self.adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        let mut new_path = path.clone();
                        new_path.push(neighbor);
                        node_info.insert(neighbor, (depth + 1, new_path.clone()));
                        queue.push_back((neighbor, depth + 1, new_path));
                    }
                }
            }
        }

        // Compute hybrid scores for all visited nodes with embeddings
        let mut results: Vec<HybridResult> = node_info
            .iter()
            .filter_map(|(&node_id, (graph_dist, path))| {
                // Get embedding for this node from authoritative storage
                let node = self.nodes.get(&node_id)?;
                let embedding = if node.embedding.is_empty() {
                    return None;
                } else {
                    node.embedding.as_slice()
                };

                // Skip if dimensions don't match
                if embedding.len() != query_embedding.len() {
                    return None;
                }

                // Compute vector distance
                let vec_dist = l2_distance(query_embedding, embedding);

                // Compute hybrid score
                let score = compute_hybrid_score(vec_dist, *graph_dist, &params);

                Some(HybridResult::new(
                    node_id,
                    score,
                    vec_dist,
                    *graph_dist,
                    path.clone(),
                ))
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top k
        results.truncate(k);
        results
    }

    /// Records an agent decision to the database.
    ///
    /// The decision is written to the WAL for durability and stored
    /// in memory for querying.
    ///
    /// # Arguments
    ///
    /// * `record` - The decision record to store
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use barq_graphdb::storage::{BarqGraphDb, DbOptions};
    /// use barq_graphdb::agent::DecisionRecord;
    /// use std::path::PathBuf;
    ///
    /// let opts = DbOptions::new(PathBuf::from("./my_db"));
    /// let mut db = BarqGraphDb::open(opts).unwrap();
    ///
    /// let decision = DecisionRecord::new(1, 42, 100, vec![100, 101], 0.95);
    /// db.record_decision(decision).unwrap();
    /// ```
    pub fn record_decision(&mut self, record: DecisionRecord) -> Result<()> {
        let wal_record = WalRecord::Decision {
            data: record.clone(),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&wal_record)
            .with_context(|| "Failed to serialize decision to JSON")?;

        // Append to WAL
        writeln!(self.wal, "{}", json).with_context(|| "Failed to write decision to WAL")?;

        // Flush to ensure durability
        self.wal.flush().with_context(|| "Failed to flush WAL")?;

        // Add to in-memory storage
        self.decisions.push(record);

        Ok(())
    }

    /// Lists all decisions for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - ID of the agent to filter by
    ///
    /// # Returns
    ///
    /// A vector of references to decision records for the specified agent.
    pub fn list_decisions_for_agent(&self, agent_id: u64) -> Vec<&DecisionRecord> {
        self.decisions
            .iter()
            .filter(|d| d.agent_id == agent_id)
            .collect()
    }

    /// Lists all decisions in the database.
    ///
    /// # Returns
    ///
    /// A vector of references to all decision records.
    pub fn list_all_decisions(&self) -> Vec<&DecisionRecord> {
        self.decisions.iter().collect()
    }

    /// Returns the total number of decisions in the database.
    pub fn decision_count(&self) -> usize {
        self.decisions.len()
    }

    /// Gets a decision by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The decision ID to look up
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the decision if found.
    pub fn get_decision(&self, id: u64) -> Option<&DecisionRecord> {
        self.decisions.iter().find(|d| d.id == id)
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
