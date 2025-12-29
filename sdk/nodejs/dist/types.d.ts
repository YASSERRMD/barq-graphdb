/**
 * Type definitions for Barq-GraphDB SDK.
 */
/**
 * Represents a node in the graph.
 */
export interface Node {
    id: number;
    label: string;
    embedding?: number[];
    agent_id?: number;
    rule_tags?: string[];
    timestamp?: number;
    has_embedding?: boolean;
}
/**
 * Represents a directed edge between nodes.
 */
export interface Edge {
    from: number;
    to: number;
    edge_type: string;
}
/**
 * Parameters for hybrid queries.
 */
export interface HybridParams {
    alpha: number;
    beta: number;
}
/**
 * Result from a hybrid query.
 */
export interface HybridResult {
    id: number;
    score: number;
    vector_distance: number;
    graph_distance: number;
    path: number[];
}
/**
 * Represents an agent decision record.
 */
export interface Decision {
    id?: number;
    agent_id: number;
    root_node: number;
    path: number[];
    score: number;
    notes?: string;
    created_at?: number;
}
/**
 * Database statistics.
 */
export interface Stats {
    node_count: number;
    edge_count: number;
    vector_count: number;
    decision_count: number;
}
/**
 * Health check response.
 */
export interface HealthResponse {
    status: string;
    version: string;
}
/**
 * Options for creating a client.
 */
export interface ClientOptions {
    baseUrl: string;
    timeout?: number;
}
/**
 * Hybrid query request.
 */
export interface HybridQueryRequest {
    start: number;
    query_embedding: number[];
    max_hops: number;
    k: number;
    alpha: number;
    beta: number;
}
/**
 * Set embedding request.
 */
export interface SetEmbeddingRequest {
    id: number;
    embedding: number[];
}
