"""
Barq-GraphDB Client

HTTP client for interacting with Barq-GraphDB REST API.
"""

from typing import List, Optional
import requests

from .models import Node, Edge, Decision, HybridResult, HybridParams, Stats

try:
    import grpc
    from .proto import barq_pb2, barq_pb2_grpc
    GRPC_AVAILABLE = True
except ImportError:
    GRPC_AVAILABLE = False
    grpc = None


class BarqError(Exception):
    """Exception raised for Barq-GraphDB API errors."""
    
    def __init__(self, message: str, status_code: int = 0):
        self.message = message
        self.status_code = status_code
        super().__init__(self.message)


class BarqClient:
    """
    Client for Barq-GraphDB REST API.
    
    Example:
        >>> client = BarqClient("http://localhost:3000")
        >>> client.create_node(Node(id=1, label="User"))
        >>> nodes = client.list_nodes()
    """
    
    def __init__(self, base_url: str = "http://localhost:3000", grpc_address: str = "localhost:50051", use_grpc: bool = False, timeout: int = 30):
        """
        Initialize the Barq-GraphDB client.
        
        Args:
            base_url: Base URL of the Barq-GraphDB server (HTTP).
            grpc_address: Address of the gRPC server (e.g. "localhost:50051").
            use_grpc: Whether to use gRPC for supported operations.
            timeout: Request timeout in seconds.
        """
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.use_grpc = use_grpc
        self.grpc_address = grpc_address
        
        if self.use_grpc:
            if not GRPC_AVAILABLE:
                raise ImportError("gRPC libraries not found or protos not generated. Install 'grpcio' and run codegen.")
            self._channel = grpc.insecure_channel(self.grpc_address)
            self._stub = barq_pb2_grpc.BarqServiceStub(self._channel)

        self._session = requests.Session()
        self._session.headers.update({
            "Content-Type": "application/json",
            "Accept": "application/json",
        })

    def _request(self, method: str, endpoint: str, json: dict = None, params: dict = None) -> dict:
        """Make an HTTP request to the API."""
        url = f"{self.base_url}{endpoint}"
        try:
            response = self._session.request(
                method=method,
                url=url,
                json=json,
                params=params,
                timeout=self.timeout,
            )
            response.raise_for_status()
            return response.json()
        except requests.exceptions.HTTPError as e:
            try:
                error_data = e.response.json()
                raise BarqError(error_data.get("error", str(e)), e.response.status_code)
            except ValueError:
                raise BarqError(str(e), e.response.status_code if e.response else 0)
        except requests.exceptions.RequestException as e:
            raise BarqError(f"Connection error: {e}")

    def health(self) -> dict:
        """
        Check server health.
        
        Returns:
            Health status dictionary with 'status' and 'version'.
        """
        return self._request("GET", "/health")

    def stats(self) -> Stats:
        """
        Get database statistics.
        
        Returns:
            Stats object with node, edge, vector, and decision counts.
        """
        data = self._request("GET", "/stats")
        return Stats.from_dict(data)

    # Node operations
    
    def create_node(self, node: Node) -> int:
        """
        Create a new node.
        
        Args:
            node: Node object to create.
            
        Returns:
            The created node's ID.
        """

        if self.use_grpc:
            try:
                props = {k: str(v) for k, v in node.properties.items()} if node.properties else {}
                proto_node = barq_pb2.NodeProto(
                    id=node.id,
                    label=node.label,
                    properties=props
                )
                req = barq_pb2.CreateNodeRequest(node=proto_node)
                resp = self._stub.CreateNode(req)
                return resp.node.id
            except grpc.RpcError as e:
                raise BarqError(f"gRPC error: {e.details()}", status_code=500)

        data = self._request("POST", "/nodes", json=node.to_dict())
        return data.get("node_id", node.id)

    def list_nodes(self) -> List[Node]:
        """
        List all nodes in the database.
        
        Returns:
            List of Node objects.
        """
        data = self._request("GET", "/nodes")
        return [Node.from_dict(n) for n in data.get("nodes", [])]

    # Edge operations
    
    def create_edge(self, edge: Edge) -> None:
        """
        Create a new edge between nodes.
        
        Args:
            edge: Edge object to create.
        """
        self._request("POST", "/edges", json=edge.to_dict())

    def add_edge(self, from_node: int, to_node: int, edge_type: str) -> None:
        """
        Convenience method to add an edge.
        
        Args:
            from_node: Source node ID.
            to_node: Target node ID.
            edge_type: Type/label of the edge.
        """
        self.create_edge(Edge(from_node, to_node, edge_type))

    # Embedding operations
    
    def set_embedding(self, node_id: int, embedding: List[float]) -> None:
        """
        Set the embedding vector for a node.
        
        Args:
            node_id: ID of the node.
            embedding: Vector embedding.
        """

        if self.use_grpc:
            try:
                req = barq_pb2.SetEmbeddingRequest(
                    id=node_id,
                    embedding=embedding
                )
                self._stub.SetEmbedding(req)
                return
            except grpc.RpcError as e:
                raise BarqError(f"gRPC error: {e.details()}", status_code=500)

        self._request("POST", "/embeddings", json={
            "id": node_id,
            "embedding": embedding,
        })

    # Query operations
    
    def hybrid_query(
        self,
        start: int,
        query_embedding: List[float],
        max_hops: int = 3,
        k: int = 10,
        params: Optional[HybridParams] = None,
    ) -> List[HybridResult]:
        """
        Perform a hybrid query combining vector similarity and graph distance.
        
        Args:
            start: Starting node ID for BFS traversal.
            query_embedding: Query vector for similarity comparison.
            max_hops: Maximum BFS depth.
            k: Number of top results to return.
            params: Hybrid scoring parameters (alpha, beta weights).
            
        Returns:
            List of HybridResult objects sorted by score.
        """
        if params is None:
            params = HybridParams()
        
        payload = {
            "start": start,
            "query_embedding": query_embedding,
            "max_hops": max_hops,
            "k": k,
            **params.to_dict(),
        }
        data = self._request("POST", "/query/hybrid", json=payload)
        return [HybridResult.from_dict(r) for r in data.get("results", [])]

    # Decision operations
    
    def record_decision(self, decision: Decision) -> Decision:
        """
        Record an agent decision.
        
        Args:
            decision: Decision object to record.
            
        Returns:
            The recorded decision with assigned ID and timestamp.
        """
        data = self._request("POST", "/decisions", json=decision.to_dict())
        return Decision.from_dict(data.get("decision", {}))

    def list_decisions(self, agent_id: int) -> List[Decision]:
        """
        List all decisions for a specific agent.
        
        Args:
            agent_id: ID of the agent to filter by.
            
        Returns:
            List of Decision objects.
        """
        data = self._request("GET", "/decisions", params={"agent_id": agent_id})
        return [Decision.from_dict(d) for d in data.get("decisions", [])]

    def close(self) -> None:
        """Close the client session."""
        self._session.close()

    def __enter__(self) -> "BarqClient":
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        self.close()
