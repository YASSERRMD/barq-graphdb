use crate::storage::BarqGraphDb;
use crate::{Node, NodeId};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub mod barq_rpc {
    tonic::include_proto!("barq");
}

use barq_rpc::barq_service_server::BarqService;
use barq_rpc::{
    EdgeProto, EmbeddingProto, Empty, HealthCheckResponse, HybridQueryRequest, HybridQueryResponse,
    HybridResultProto, NodeIdProto, NodeProto, Result as RpcResult,
};

pub struct MyBarqService {
    db: Arc<Mutex<BarqGraphDb>>,
}

impl MyBarqService {
    pub fn new(db: Arc<Mutex<BarqGraphDb>>) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl BarqService for MyBarqService {
    async fn health_check(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: "ok".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }))
    }

    async fn create_node(
        &self,
        request: Request<NodeProto>,
    ) -> Result<Response<RpcResult>, Status> {
        let req = request.into_inner();
        let mut node = Node::new(req.id, req.label);
        node.embedding = req.embedding;

        let mut db = self.db.lock().await;
        match db.append_node(node) {
            Ok(_) => Ok(Response::new(RpcResult {
                success: true,
                error: "".into(),
            })),
            Err(e) => Ok(Response::new(RpcResult {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn get_node(&self, request: Request<NodeIdProto>) -> Result<Response<NodeProto>, Status> {
        let req = request.into_inner();
        let db = self.db.lock().await;

        if let Some(node) = db.get_node(req.id) {
            let edges = node
                .edges
                .iter()
                .map(|e| EdgeProto {
                    from: e.from,
                    to: e.to,
                    r#type: e.edge_type.clone(),
                })
                .collect();

            Ok(Response::new(NodeProto {
                id: node.id,
                label: node.label.clone(),
                embedding: node.embedding.clone(),
                edges,
            }))
        } else {
            Err(Status::not_found("Node not found"))
        }
    }

    async fn create_edge(
        &self,
        request: Request<EdgeProto>,
    ) -> Result<Response<RpcResult>, Status> {
        let req = request.into_inner();
        let mut db = self.db.lock().await;

        match db.add_edge(req.from, req.to, &req.r#type) {
            Ok(_) => Ok(Response::new(RpcResult {
                success: true,
                error: "".into(),
            })),
            Err(e) => Ok(Response::new(RpcResult {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn set_embedding(
        &self,
        request: Request<EmbeddingProto>,
    ) -> Result<Response<RpcResult>, Status> {
        let req = request.into_inner();
        let mut db = self.db.lock().await;

        match db.set_embedding(req.id, req.vec) {
            Ok(_) => Ok(Response::new(RpcResult {
                success: true,
                error: "".into(),
            })),
            Err(e) => Ok(Response::new(RpcResult {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn hybrid_query(
        &self,
        request: Request<HybridQueryRequest>,
    ) -> Result<Response<HybridQueryResponse>, Status> {
        let req = request.into_inner();
        let db = self.db.lock().await;

        let params = crate::hybrid::HybridParams::new(req.alpha, req.beta);
        let results = db.hybrid_query(
            &req.query_embedding,
            req.start_node as NodeId,
            req.max_hops as usize,
            req.k as usize,
            params,
        );

        let proto_results = results
            .into_iter()
            .map(|r| HybridResultProto {
                id: r.id,
                score: r.score,
                path: r.path,
            })
            .collect();

        Ok(Response::new(HybridQueryResponse {
            results: proto_results,
        }))
    }
}
