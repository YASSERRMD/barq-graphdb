//! Barq-GraphDB CLI tool.
//!
//! This binary provides a command-line interface for interacting with
//! Barq-GraphDB databases. It supports essential operations like
//! initializing databases, adding nodes, and querying data.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::json;

use barq_graphdb::agent::DecisionRecord;
use barq_graphdb::hybrid::HybridParams;
use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use barq_graphdb::Node;

/// Barq-GraphDB command-line interface.
///
/// A production-grade graph+vector database for AI agents.
#[derive(Parser)]
#[command(name = "barqg")]
#[command(author = "YASSERRMD")]
#[command(version = "0.0.1")]
#[command(about = "Graph+Vector database for AI agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands.
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new database at the specified path.
    Init {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,
    },

    /// Add a new node to the database.
    AddNode {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,

        /// Unique node ID.
        #[arg(long)]
        id: u64,

        /// Human-readable label for the node.
        #[arg(long)]
        label: String,
    },

    /// List all nodes in the database.
    ListNodes {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,
    },

    /// Add a directed edge between two nodes.
    AddEdge {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,

        /// Source node ID.
        #[arg(long)]
        from: u64,

        /// Target node ID.
        #[arg(long)]
        to: u64,

        /// Edge type/label.
        #[arg(long, name = "type")]
        edge_type: String,
    },

    /// List neighbors of a node.
    Neighbors {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,

        /// Node ID to get neighbors for.
        #[arg(long)]
        id: u64,
    },

    /// Perform BFS traversal from a node.
    Bfs {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,

        /// Starting node ID.
        #[arg(long)]
        start: u64,

        /// Maximum number of hops.
        #[arg(long)]
        hops: usize,
    },

    /// Set embedding for a node.
    SetEmbedding {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,

        /// Node ID to set embedding for.
        #[arg(long)]
        id: u64,

        /// Embedding vector as JSON array, e.g., '[0.1,0.2,0.3]'.
        #[arg(long)]
        vec: String,
    },

    /// Find k nearest neighbors to a query vector.
    Knn {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,

        /// Query vector as JSON array, e.g., '[0.1,0.2,0.3]'.
        #[arg(long)]
        vec: String,

        /// Number of nearest neighbors to return.
        #[arg(long)]
        k: usize,
    },

    /// Perform hybrid query combining vector similarity and graph distance.
    Hybrid {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,

        /// Starting node ID for BFS traversal.
        #[arg(long)]
        start: u64,

        /// Maximum number of hops for BFS.
        #[arg(long)]
        hops: usize,

        /// Number of top results to return.
        #[arg(long)]
        k: usize,

        /// Query vector as JSON array, e.g., '[0.1,0.2,0.3]'.
        #[arg(long)]
        vec: String,

        /// Weight for vector similarity (0.0 to 1.0).
        #[arg(long, default_value = "0.5")]
        alpha: f32,

        /// Weight for graph distance (0.0 to 1.0).
        #[arg(long, default_value = "0.5")]
        beta: f32,
    },

    /// Record an agent decision.
    RecordDecision {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,

        /// Agent ID that made the decision.
        #[arg(long)]
        agent_id: u64,

        /// Root/starting node for the decision.
        #[arg(long)]
        root: u64,

        /// Decision path as JSON array, e.g., '[1,2,3]'.
        #[arg(long)]
        decision_path: String,

        /// Confidence score for the decision.
        #[arg(long)]
        score: f32,

        /// Optional notes about the decision.
        #[arg(long)]
        notes: Option<String>,
    },

    /// List decisions for an agent.
    ListDecisions {
        /// Path to the database directory.
        #[arg(long)]
        path: PathBuf,

        /// Agent ID to filter by.
        #[arg(long)]
        agent_id: u64,
    },
}

/// Entry point for the CLI application.
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => init_database(path),
        Commands::AddNode { path, id, label } => add_node(path, id, label),
        Commands::ListNodes { path } => list_nodes(path),
        Commands::AddEdge {
            path,
            from,
            to,
            edge_type,
        } => add_edge(path, from, to, edge_type),
        Commands::Neighbors { path, id } => neighbors(path, id),
        Commands::Bfs { path, start, hops } => bfs(path, start, hops),
        Commands::SetEmbedding { path, id, vec } => set_embedding(path, id, vec),
        Commands::Knn { path, vec, k } => knn(path, vec, k),
        Commands::Hybrid {
            path,
            start,
            hops,
            k,
            vec,
            alpha,
            beta,
        } => hybrid(path, start, hops, k, vec, alpha, beta),
        Commands::RecordDecision {
            path,
            agent_id,
            root,
            decision_path,
            score,
            notes,
        } => record_decision(path, agent_id, root, decision_path, score, notes),
        Commands::ListDecisions { path, agent_id } => list_decisions(path, agent_id),
    }
}

/// Initializes a new database at the specified path.
///
/// Creates the database directory and initializes an empty WAL file.
fn init_database(path: PathBuf) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let _db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to initialize database at {:?}", path))?;

    let output = json!({
        "status": "ok",
        "message": format!("Database initialized at {:?}", path)
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Adds a new node to the database.
///
/// Creates a node with the given ID and label, using the current
/// timestamp and empty values for optional fields.
fn add_node(path: PathBuf, id: u64, label: String) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let mut db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    let node = Node::new(id, label.clone());
    db.append_node(node)
        .with_context(|| format!("Failed to add node with id {}", id))?;

    let output = json!({
        "status": "ok",
        "node": {
            "id": id,
            "label": label
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Lists all nodes in the database.
///
/// Outputs a JSON array containing basic information about each node.
fn list_nodes(path: PathBuf) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    let nodes: Vec<_> = db
        .list_nodes()
        .iter()
        .map(|node| {
            json!({
                "id": node.id,
                "label": node.label
            })
        })
        .collect();

    let output = json!({ "nodes": nodes });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Adds a directed edge between two nodes.
fn add_edge(path: PathBuf, from: u64, to: u64, edge_type: String) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let mut db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    db.add_edge(from, to, &edge_type)
        .with_context(|| format!("Failed to add edge from {} to {}", from, to))?;

    let output = json!({
        "status": "ok",
        "edge": {
            "from": from,
            "to": to,
            "type": edge_type
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Lists neighbors of a node.
fn neighbors(path: PathBuf, id: u64) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    let neighbors = db.neighbors(id).unwrap_or(&[]);

    let output = json!({ "neighbors": neighbors });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Performs BFS traversal from a node.
fn bfs(path: PathBuf, start: u64, hops: usize) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    let result = db.bfs_hops(start, hops);

    let output = json!({ "bfs": result });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Sets embedding for a node.
fn set_embedding(path: PathBuf, id: u64, vec_str: String) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let mut db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    let embedding: Vec<f32> = serde_json::from_str(&vec_str)
        .with_context(|| format!("Failed to parse embedding vector: {}", vec_str))?;

    db.set_embedding(id, embedding.clone())
        .with_context(|| format!("Failed to set embedding for node {}", id))?;

    let output = json!({
        "status": "ok",
        "embedding": {
            "id": id,
            "dimension": embedding.len()
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Finds k nearest neighbors to a query vector.
fn knn(path: PathBuf, vec_str: String, k: usize) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    let query: Vec<f32> = serde_json::from_str(&vec_str)
        .with_context(|| format!("Failed to parse query vector: {}", vec_str))?;

    let results = db.knn_search(&query, k);

    let output = json!({
        "results": results.iter().map(|(id, dist)| {
            json!({ "id": id, "distance": dist })
        }).collect::<Vec<_>>()
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Performs hybrid query combining vector similarity and graph distance.
fn hybrid(
    path: PathBuf,
    start: u64,
    hops: usize,
    k: usize,
    vec_str: String,
    alpha: f32,
    beta: f32,
) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    let query: Vec<f32> = serde_json::from_str(&vec_str)
        .with_context(|| format!("Failed to parse query vector: {}", vec_str))?;

    let params = HybridParams::new(alpha, beta);
    let results = db.hybrid_query(&query, start, hops, k, params);

    let output = json!({
        "results": results.iter().map(|r| {
            json!({
                "id": r.id,
                "score": r.score,
                "vector_distance": r.vector_distance,
                "graph_distance": r.graph_distance,
                "path": r.path
            })
        }).collect::<Vec<_>>()
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Records an agent decision.
fn record_decision(
    path: PathBuf,
    agent_id: u64,
    root: u64,
    decision_path_str: String,
    score: f32,
    notes: Option<String>,
) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let mut db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    let decision_path: Vec<u64> = serde_json::from_str(&decision_path_str)
        .with_context(|| format!("Failed to parse decision path: {}", decision_path_str))?;

    // Generate unique decision ID based on current count
    let decision_id = db.decision_count() as u64 + 1;

    let mut record = DecisionRecord::new(decision_id, agent_id, root, decision_path, score);
    if let Some(n) = notes {
        record = record.with_notes(n);
    }

    db.record_decision(record.clone())
        .with_context(|| "Failed to record decision")?;

    let output = json!({
        "status": "ok",
        "decision": {
            "id": record.id,
            "agent_id": record.agent_id,
            "root_node": record.root_node,
            "path": record.path,
            "score": record.score,
            "created_at": record.created_at,
            "notes": record.notes
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Lists decisions for an agent.
fn list_decisions(path: PathBuf, agent_id: u64) -> Result<()> {
    let opts = DbOptions::new(path.clone());
    let db = BarqGraphDb::open(opts)
        .with_context(|| format!("Failed to open database at {:?}", path))?;

    let decisions = db.list_decisions_for_agent(agent_id);

    let output = json!({
        "decisions": decisions.iter().map(|d| {
            json!({
                "id": d.id,
                "agent_id": d.agent_id,
                "root_node": d.root_node,
                "path": d.path,
                "score": d.score,
                "created_at": d.created_at,
                "notes": d.notes
            })
        }).collect::<Vec<_>>()
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}
