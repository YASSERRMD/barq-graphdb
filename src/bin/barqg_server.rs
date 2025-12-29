//! Barq-GraphDB HTTP Server.
//!
//! This binary provides a REST API server for Barq-GraphDB,
//! exposing all database operations via JSON endpoints.

use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use barq_graphdb::api;
use barq_graphdb::storage::{BarqGraphDb, DbOptions};

/// Barq-GraphDB HTTP Server.
#[derive(Parser)]
#[command(name = "barqg_server")]
#[command(about = "Barq-GraphDB HTTP API Server")]
#[command(version)]
struct Args {
    /// Path to the database directory.
    #[arg(long)]
    path: PathBuf,

    /// Host to bind to.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on.
    #[arg(long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Open database
    let opts = DbOptions::new(args.path.clone());
    let db = match BarqGraphDb::open(opts) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    let state = Arc::new(Mutex::new(db));

    // Build router with all endpoints
    let app = Router::new()
        // Health and stats
        .route("/health", get(api::health_check))
        .route("/stats", get(api::get_stats))
        // Node operations
        .route("/nodes", get(api::list_nodes))
        .route("/nodes", post(api::create_node))
        // Edge operations
        .route("/edges", post(api::create_edge))
        // Vector operations
        .route("/embeddings", post(api::set_embedding))
        // Query operations
        .route("/query/hybrid", post(api::hybrid_query))
        // Decision operations
        .route("/decisions", get(api::list_decisions))
        .route("/decisions", post(api::record_decision))
        // Add state
        .with_state(state);

    let addr = format!("{}:{}", args.host, args.port);
    println!("Barq-GraphDB server starting on http://{}", addr);
    println!("Database path: {:?}", args.path);

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
