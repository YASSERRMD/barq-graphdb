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
}

/// Entry point for the CLI application.
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => init_database(path),
        Commands::AddNode { path, id, label } => add_node(path, id, label),
        Commands::ListNodes { path } => list_nodes(path),
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
