//! HTTP API handlers for Barq-GraphDB.
//!
//! This module provides HTTP endpoint handlers for the REST API,
//! implementing JSON request/response handling for all database operations.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::agent::DecisionRecord;
use crate::hybrid::HybridParams;
use crate::storage::BarqGraphDb;
use crate::Node;

/// Shared database state for HTTP handlers.
pub type DbState = Arc<Mutex<BarqGraphDb>>;

/// Custom error type for API responses.
#[derive(Debug)]
pub struct AppError {
    /// HTTP status code.
    pub code: StatusCode,
    /// Error message.
    pub message: String,
}

impl AppError {
    /// Creates a new API error.
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Creates a bad request error.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    /// Creates an internal server error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": self.message,
            "code": self.code.as_u16()
        });
        (self.code, Json(body)).into_response()
    }
}

// ============= Request/Response Types =============

/// Request to create a node.
#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub id: u64,
    pub label: String,
    #[serde(default)]
    pub embedding: Vec<f32>,
    #[serde(default)]
    pub agent_id: Option<u64>,
    #[serde(default)]
    pub rule_tags: Vec<String>,
}

/// Request to create an edge.
#[derive(Debug, Deserialize)]
pub struct CreateEdgeRequest {
    pub from: u64,
    pub to: u64,
    pub edge_type: String,
}

/// Request to set an embedding.
#[derive(Debug, Deserialize)]
pub struct SetEmbeddingRequest {
    pub id: u64,
    pub embedding: Vec<f32>,
}

/// Request for hybrid query.
#[derive(Debug, Deserialize)]
pub struct HybridQueryRequest {
    pub start: u64,
    pub max_hops: usize,
    pub k: usize,
    pub query_embedding: Vec<f32>,
    #[serde(default = "default_alpha")]
    pub alpha: f32,
    #[serde(default = "default_beta")]
    pub beta: f32,
}

fn default_alpha() -> f32 {
    0.5
}
fn default_beta() -> f32 {
    0.5
}

/// Request to record a decision.
#[derive(Debug, Deserialize)]
pub struct RecordDecisionRequest {
    pub agent_id: u64,
    pub root_node: u64,
    pub path: Vec<u64>,
    pub score: f32,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Query parameters for listing decisions.
#[derive(Debug, Deserialize)]
pub struct ListDecisionsQuery {
    pub agent_id: u64,
}

/// Generic success response.
#[derive(Debug, Serialize)]
pub struct SuccessResponse<T: Serialize> {
    pub status: String,
    pub data: T,
}

impl<T: Serialize> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            status: "ok".to_string(),
            data,
        }
    }
}

// ============= Handler Functions =============

/// Health check endpoint.
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Creates a new node.
pub async fn create_node(
    State(db): State<DbState>,
    Json(payload): Json<CreateNodeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut db = db.lock().await;

    let mut node = Node::new(payload.id, payload.label);
    node.embedding = payload.embedding;
    node.agent_id = payload.agent_id;
    node.rule_tags = payload.rule_tags;

    db.append_node(node)
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "ok",
            "node_id": payload.id
        })),
    ))
}

/// Creates a new edge.
pub async fn create_edge(
    State(db): State<DbState>,
    Json(payload): Json<CreateEdgeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut db = db.lock().await;

    db.add_edge(payload.from, payload.to, &payload.edge_type)
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "ok",
            "from": payload.from,
            "to": payload.to
        })),
    ))
}

/// Sets an embedding for a node.
pub async fn set_embedding(
    State(db): State<DbState>,
    Json(payload): Json<SetEmbeddingRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut db = db.lock().await;

    db.set_embedding(payload.id, payload.embedding)
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "node_id": payload.id
    })))
}

/// Performs a hybrid query.
pub async fn hybrid_query(
    State(db): State<DbState>,
    Json(payload): Json<HybridQueryRequest>,
) -> Result<impl IntoResponse, AppError> {
    let db = db.lock().await;

    let params = HybridParams::new(payload.alpha, payload.beta);
    let results = db.hybrid_query(
        &payload.query_embedding,
        payload.start,
        payload.max_hops,
        payload.k,
        params,
    );

    let response: Vec<_> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "score": r.score,
                "vector_distance": r.vector_distance,
                "graph_distance": r.graph_distance,
                "path": r.path
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "results": response
    })))
}

/// Records a decision.
pub async fn record_decision(
    State(db): State<DbState>,
    Json(payload): Json<RecordDecisionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut db = db.lock().await;

    let decision_id = db.decision_count() as u64 + 1;
    let mut record = DecisionRecord::new(
        decision_id,
        payload.agent_id,
        payload.root_node,
        payload.path,
        payload.score,
    );

    if let Some(notes) = payload.notes {
        record = record.with_notes(notes);
    }

    db.record_decision(record.clone())
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "ok",
            "decision": {
                "id": record.id,
                "agent_id": record.agent_id,
                "root_node": record.root_node,
                "path": record.path,
                "score": record.score,
                "created_at": record.created_at
            }
        })),
    ))
}

/// Lists decisions for an agent.
pub async fn list_decisions(
    State(db): State<DbState>,
    Query(query): Query<ListDecisionsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let db = db.lock().await;

    let decisions = db.list_decisions_for_agent(query.agent_id);

    let response: Vec<_> = decisions
        .iter()
        .map(|d| {
            serde_json::json!({
                "id": d.id,
                "agent_id": d.agent_id,
                "root_node": d.root_node,
                "path": d.path,
                "score": d.score,
                "created_at": d.created_at,
                "notes": d.notes
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "decisions": response
    })))
}

/// Lists all nodes.
pub async fn list_nodes(State(db): State<DbState>) -> Result<impl IntoResponse, AppError> {
    let db = db.lock().await;

    let nodes: Vec<_> = db
        .list_nodes()
        .iter()
        .map(|n| {
            serde_json::json!({
                "id": n.id,
                "label": n.label,
                "has_embedding": !n.embedding.is_empty(),
                "agent_id": n.agent_id,
                "timestamp": n.timestamp
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "nodes": nodes,
        "count": nodes.len()
    })))
}

/// Gets database stats.
pub async fn get_stats(State(db): State<DbState>) -> Result<impl IntoResponse, AppError> {
    let db = db.lock().await;

    Ok(Json(serde_json::json!({
        "node_count": db.node_count(),
        "edge_count": db.edge_count(),
        "vector_count": db.vector_count(),
        "decision_count": db.decision_count()
    })))
}
