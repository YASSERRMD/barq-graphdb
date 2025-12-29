/**
 * Barq-GraphDB Client
 *
 * HTTP client for interacting with Barq-GraphDB REST API.
 */

import type {
    Node,
    Edge,
    HybridParams,
    HybridResult,
    Decision,
    Stats,
    HealthResponse,
    ClientOptions,
    HybridQueryRequest,
    SetEmbeddingRequest,
} from './types.js';

/**
 * Error thrown for Barq-GraphDB API errors.
 */
export class BarqError extends Error {
    public readonly statusCode: number;

    constructor(message: string, statusCode: number = 0) {
        super(message);
        this.name = 'BarqError';
        this.statusCode = statusCode;
    }
}

/**
 * Default hybrid parameters.
 */
export const defaultHybridParams: HybridParams = {
    alpha: 0.5,
    beta: 0.5,
};

/**
 * Client for Barq-GraphDB REST API.
 */
export class BarqClient {
    private readonly baseUrl: string;
    private readonly timeout: number;

    /**
     * Creates a new Barq-GraphDB client.
     *
     * @param options - Client options or base URL string.
     */
    constructor(options: ClientOptions | string) {
        if (typeof options === 'string') {
            this.baseUrl = options.replace(/\/$/, '');
            this.timeout = 30000;
        } else {
            this.baseUrl = options.baseUrl.replace(/\/$/, '');
            this.timeout = options.timeout ?? 30000;
        }
    }

    private async request<T>(
        method: string,
        endpoint: string,
        body?: unknown
    ): Promise<T> {
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
                } catch {
                    // Use text as-is
                }
                throw new BarqError(message, response.status);
            }

            return (await response.json()) as T;
        } catch (error) {
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
    async health(): Promise<HealthResponse> {
        return this.request<HealthResponse>('GET', '/health');
    }

    /**
     * Gets database statistics.
     */
    async stats(): Promise<Stats> {
        return this.request<Stats>('GET', '/stats');
    }

    /**
     * Creates a new node.
     */
    async createNode(node: Node): Promise<{ status: string; node_id: number }> {
        return this.request('POST', '/nodes', node);
    }

    /**
     * Lists all nodes.
     */
    async listNodes(): Promise<Node[]> {
        const response = await this.request<{ nodes: Node[]; count: number }>(
            'GET',
            '/nodes'
        );
        return response.nodes;
    }

    /**
     * Creates a new edge.
     */
    async createEdge(edge: Edge): Promise<void> {
        await this.request('POST', '/edges', edge);
    }

    /**
     * Adds an edge between nodes.
     */
    async addEdge(from: number, to: number, edgeType: string): Promise<void> {
        await this.createEdge({ from, to, edge_type: edgeType });
    }

    /**
     * Sets the embedding for a node.
     */
    async setEmbedding(nodeId: number, embedding: number[]): Promise<void> {
        const request: SetEmbeddingRequest = {
            id: nodeId,
            embedding,
        };
        await this.request('POST', '/embeddings', request);
    }

    /**
     * Performs a hybrid query combining vector similarity and graph distance.
     */
    async hybridQuery(
        start: number,
        queryEmbedding: number[],
        maxHops: number = 3,
        k: number = 10,
        params: HybridParams = defaultHybridParams
    ): Promise<HybridResult[]> {
        const request: HybridQueryRequest = {
            start,
            query_embedding: queryEmbedding,
            max_hops: maxHops,
            k,
            alpha: params.alpha,
            beta: params.beta,
        };
        const response = await this.request<{ results: HybridResult[] }>(
            'POST',
            '/query/hybrid',
            request
        );
        return response.results;
    }

    /**
     * Records an agent decision.
     */
    async recordDecision(decision: Decision): Promise<Decision> {
        const response = await this.request<{ status: string; decision: Decision }>(
            'POST',
            '/decisions',
            decision
        );
        return response.decision;
    }

    /**
     * Lists all decisions for a specific agent.
     */
    async listDecisions(agentId: number): Promise<Decision[]> {
        const response = await this.request<{ decisions: Decision[] }>(
            'GET',
            `/decisions?agent_id=${agentId}`
        );
        return response.decisions;
    }
}
