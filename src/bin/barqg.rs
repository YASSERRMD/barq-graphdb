//! Barq-GraphDB CLI tool.
//!
//! This binary provides a command-line interface for interacting with
//! Barq-GraphDB databases. It supports essential operations like
//! initializing databases, adding nodes, and querying data.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::json;

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
