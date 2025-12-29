"use strict";
/**
 * Barq-GraphDB Client
 *
 * HTTP client for interacting with Barq-GraphDB REST API.
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.BarqClient = exports.defaultHybridParams = exports.BarqError = void 0;
/**
 * Error thrown for Barq-GraphDB API errors.
 */
class BarqError extends Error {
    statusCode;
    constructor(message, statusCode = 0) {
        super(message);
        this.name = 'BarqError';
        this.statusCode = statusCode;
    }
}
exports.BarqError = BarqError;
/**
 * Default hybrid parameters.
 */
exports.defaultHybridParams = {
    alpha: 0.5,
    beta: 0.5,
};
/**
 * Client for Barq-GraphDB REST API.
 */
class BarqClient {
    baseUrl;
    timeout;
    /**
     * Creates a new Barq-GraphDB client.
     *
     * @param options - Client options or base URL string.
     */
    constructor(options) {
        if (typeof options === 'string') {
            this.baseUrl = options.replace(/\/$/, '');
            this.timeout = 30000;
        }
        else {
            this.baseUrl = options.baseUrl.replace(/\/$/, '');
            this.timeout = options.timeout ?? 30000;
        }
    }
    async request(method, endpoint, body) {
        const url = `${this.baseUrl}${endpoint}`;
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.timeout);
        try {
            const response = await fetch(url, {
                method,
                headers: {
                    'Content-Type': 'application/json',
                    Accept: 'application/json',
                },
                body: body ? JSON.stringify(body) : undefined,
                signal: controller.signal,
            });
            clearTimeout(timeoutId);
            if (!response.ok) {
                const text = await response.text();
                let message = text;
                try {
                    const json = JSON.parse(text);
                    message = json.error || text;
                }
                catch {
                    // Use text as-is
                }
                throw new BarqError(message, response.status);
            }
            return (await response.json());
        }
        catch (error) {
            clearTimeout(timeoutId);
            if (error instanceof BarqError) {
                throw error;
            }
            if (error instanceof Error && error.name === 'AbortError') {
                throw new BarqError('Request timeout');
            }
            throw new BarqError(`Request failed: ${error}`);
        }
    }
    /**
     * Checks server health.
     */
    async health() {
        return this.request('GET', '/health');
    }
    /**
     * Gets database statistics.
     */
    async stats() {
        return this.request('GET', '/stats');
    }
    /**
     * Creates a new node.
     */
    async createNode(node) {
        return this.request('POST', '/nodes', node);
    }
    /**
     * Lists all nodes.
     */
    async listNodes() {
        const response = await this.request('GET', '/nodes');
        return response.nodes;
    }
    /**
     * Creates a new edge.
     */
    async createEdge(edge) {
        await this.request('POST', '/edges', edge);
    }
    /**
     * Adds an edge between nodes.
     */
    async addEdge(from, to, edgeType) {
        await this.createEdge({ from, to, edge_type: edgeType });
    }
    /**
     * Sets the embedding for a node.
     */
    async setEmbedding(nodeId, embedding) {
        const request = {
            id: nodeId,
            embedding,
        };
        await this.request('POST', '/embeddings', request);
    }
    /**
     * Performs a hybrid query combining vector similarity and graph distance.
     */
    async hybridQuery(start, queryEmbedding, maxHops = 3, k = 10, params = exports.defaultHybridParams) {
        const request = {
            start,
            query_embedding: queryEmbedding,
            max_hops: maxHops,
            k,
            alpha: params.alpha,
            beta: params.beta,
        };
        const response = await this.request('POST', '/query/hybrid', request);
        return response.results;
    }
    /**
     * Records an agent decision.
     */
    async recordDecision(decision) {
        const response = await this.request('POST', '/decisions', decision);
        return response.decision;
    }
    /**
     * Lists all decisions for a specific agent.
     */
    async listDecisions(agentId) {
        const response = await this.request('GET', `/decisions?agent_id=${agentId}`);
        return response.decisions;
    }
}
exports.BarqClient = BarqClient;
