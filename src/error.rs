//! Error types for Barq-GraphDB operations.
//!
//! This module defines custom error types using `thiserror` for
//! type-safe error handling throughout the database.

use thiserror::Error;

/// Errors that can occur during database operations.
#[derive(Error, Debug)]
pub enum BarqError {
    /// Error occurred during I/O operations (file read/write).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error occurred during JSON serialization/deserialization.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Requested node was not found in the database.
    #[error("Node not found: {0}")]
    NodeNotFound(u64),

    /// Attempted to add a node that already exists.
    #[error("Node already exists: {0}")]
    NodeAlreadyExists(u64),

    /// Error occurred during WAL (Write-Ahead Log) operations.
    #[error("WAL error: {0}")]
    WalError(String),

    /// Invalid operation or parameter.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Database is in an invalid state.
    #[error("Database corrupt: {0}")]
    DatabaseCorrupt(String),
}

/// Result type alias for Barq operations.
pub type BarqResult<T> = Result<T, BarqError>;
