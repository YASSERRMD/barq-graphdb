/**
 * Barq-GraphDB Client
 *
 * HTTP client for interacting with Barq-GraphDB REST API.
 */
import type { Node, Edge, HybridParams, HybridResult, Decision, Stats, HealthResponse, ClientOptions } from './types.js';
/**
 * Error thrown for Barq-GraphDB API errors.
 */
export declare class BarqError extends Error {
    readonly statusCode: number;
    constructor(message: string, statusCode?: number);
}
/**
 * Default hybrid parameters.
 */
export declare const defaultHybridParams: HybridParams;
/**
 * Client for Barq-GraphDB REST API.
 */
export declare class BarqClient {
    private readonly baseUrl;
    private readonly timeout;
    /**
     * Creates a new Barq-GraphDB client.
     *
     * @param options - Client options or base URL string.
     */
    constructor(options: ClientOptions | string);
    private request;
    /**
     * Checks server health.
     */
    health(): Promise<HealthResponse>;
    /**
     * Gets database statistics.
     */
    stats(): Promise<Stats>;
    /**
     * Creates a new node.
     */
    createNode(node: Node): Promise<{
        status: string;
        node_id: number;
    }>;
    /**
     * Lists all nodes.
     */
    listNodes(): Promise<Node[]>;
    /**
     * Creates a new edge.
     */
    createEdge(edge: Edge): Promise<void>;
    /**
     * Adds an edge between nodes.
     */
    addEdge(from: number, to: number, edgeType: string): Promise<void>;
    /**
     * Sets the embedding for a node.
     */
    setEmbedding(nodeId: number, embedding: number[]): Promise<void>;
    /**
     * Performs a hybrid query combining vector similarity and graph distance.
     */
    hybridQuery(start: number, queryEmbedding: number[], maxHops?: number, k?: number, params?: HybridParams): Promise<HybridResult[]>;
    /**
     * Records an agent decision.
     */
    recordDecision(decision: Decision): Promise<Decision>;
    /**
     * Lists all decisions for a specific agent.
     */
    listDecisions(agentId: number): Promise<Decision[]>;
}
