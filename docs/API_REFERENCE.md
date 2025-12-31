# Barq-GraphDB API Reference

Complete reference documentation for the Barq-GraphDB HTTP and gRPC APIs.

**Version**: v0.8.0  
**HTTP Base URL**: `http://localhost:8080`  
**gRPC Address**: `localhost:50051`

---

## Quick Start

### Docker

```bash
docker run -d -p 8080:8080 -p 50051:50051 yasserrmd/barq-graphdb:latest
```

### From Binary

```bash
./barqg_server --path ./data --host 0.0.0.0 --port 8080 --grpc-port 50051
```

---

## HTTP REST API

All HTTP endpoints use JSON for request and response bodies.

### Headers

| Header | Value | Required |
|--------|-------|----------|
| `Content-Type` | `application/json` | Yes (for POST) |
| `Accept` | `application/json` | Optional |

### Latency SLA
- **Typical Latency**: 35-60 μs (local loopback)
- **SLA Guarantee**: < 5ms for single-node operations (99th percentile)

---

### Health & Stats

#### GET /health

Check server health and version.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.8.0"
}
```

#### GET /stats

Get database statistics.

**Response:**
```json
{
  "node_count": 1000,
  "edge_count": 5000,
  "vector_count": 800,
  "decision_count": 150
}
```

---

### Node Operations

#### POST /nodes

Create a new node.

**Request:**
```json
{
  "id": 1,
  "label": "User",
  "properties": {
    "name": "John Doe",
    "role": "admin"
  },
  "embedding": [0.1, 0.2, 0.3, 0.4]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | integer | Yes | Unique node identifier |
| `label` | string | Yes | Node label/type |
| `properties` | object | No | Key-value metadata |
| `embedding` | float[] | No | Vector embedding |

**Response:**
```json
{
  "status": "ok",
  "node_id": 1
}
```

#### GET /nodes

List all nodes.

**Response:**
```json
{
  "nodes": [
    {
      "id": 1,
      "label": "User",
      "properties": {"name": "John"},
      "embedding": [0.1, 0.2, 0.3]
    }
  ],
  "count": 100
}
```

#### GET /nodes/{id}

Get a specific node by ID.

**Response:**
```json
{
  "id": 1,
  "label": "User",
  "properties": {"name": "John"},
  "embedding": [0.1, 0.2, 0.3],
  "edges": [2, 5],
  "timestamp": 1234567890
}
```

---

### Edge Operations

#### POST /edges

Create a directed edge between nodes.

**Request:**
```json
{
  "from": 1,
  "to": 2,
  "edge_type": "KNOWS"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `from` | integer | Yes | Source node ID |
| `to` | integer | Yes | Target node ID |
| `edge_type` | string | Yes | Edge type/label |

**Response:**
```json
{
  "status": "ok",
  "from": 1,
  "to": 2
}
```

---

### Embedding Operations

#### POST /embeddings

Set or update a node's vector embedding.

**Request:**
```json
{
  "id": 1,
  "embedding": [0.1, 0.2, 0.3, 0.4, 0.5]
}
```

**Response:**
```json
{
  "status": "ok",
  "node_id": 1
}
```

---

### Query Operations

#### POST /query/hybrid

Execute a hybrid query combining vector similarity and graph distance.

**Request:**
```json
{
  "start": 1,
  "query_embedding": [0.1, 0.2, 0.3, 0.4],
  "max_hops": 3,
  "k": 10,
  "alpha": 0.7,
  "beta": 0.3
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `start` | integer | Yes | - | Starting node ID for graph traversal |
| `query_embedding` | float[] | Yes | - | Query vector for similarity search |
| `max_hops` | integer | No | 3 | Maximum BFS depth |
| `k` | integer | No | 10 | Number of results to return |
| `alpha` | float | No | 0.5 | Weight for vector similarity (0.0-1.0) |
| `beta` | float | No | 0.5 | Weight for graph proximity (0.0-1.0) |

**Response:**
```json
{
  "results": [
    {
      "id": 5,
      "score": 0.92,
      "vector_distance": 0.15,
      "graph_distance": 2,
      "path": [1, 3, 5]
    }
  ]
}
```

**Scoring Formula:**
```
score = alpha * (1 - normalized_vector_distance) + beta * (1 / (1 + graph_distance))
```

---

### Decision Audit Operations

#### POST /decisions

Record an agent decision for audit trail.

**Request:**
```json
{
  "agent_id": 42,
  "root_node": 100,
  "path": [100, 101, 102, 105],
  "score": 0.95,
  "notes": "Selected based on similarity to user query"
}
```

**Response:**
```json
{
  "decision": {
    "id": 1,
    "agent_id": 42,
    "root_node": 100,
    "path": [100, 101, 102, 105],
    "score": 0.95,
    "timestamp": "2024-12-31T12:00:00Z"
  }
}
```

#### GET /decisions?agent_id={id}

List decisions for a specific agent.

**Response:**
```json
{
  "decisions": [...]
}
```

---

## gRPC API

The gRPC API provides high-performance binary communication, ideal for high-throughput embedding updates and real-time queries.

### Connection

```
grpc://localhost:50051
```

### Proto File Location

```
proto/barq.proto
```

### Service Definition

```protobuf
syntax = "proto3";
package barq;

service BarqService {
  rpc HealthCheck(HealthRequest) returns (HealthResponse);
  rpc CreateNode(CreateNodeRequest) returns (CreateNodeResponse);
  rpc GetNode(GetNodeRequest) returns (GetNodeResponse);
  rpc CreateEdge(CreateEdgeRequest) returns (CreateEdgeResponse);
  rpc SetEmbedding(SetEmbeddingRequest) returns (SetEmbeddingResponse);
  rpc HybridQuery(HybridQueryRequest) returns (HybridQueryResponse);
}
```

### Message Types

#### NodeProto
```protobuf
message NodeProto {
  uint64 id = 1;
  string label = 2;
  map<string, string> properties = 3;
  repeated EdgeProto edges = 4;
}
```

#### EdgeProto
```protobuf
message EdgeProto {
  uint64 to = 1;
  string type = 2;
}
```

#### HybridQueryRequest
```protobuf
message HybridQueryRequest {
  uint64 start_node_id = 1;
  repeated float query_embedding = 2;
  uint32 max_hops = 3;
  uint32 k = 4;
  float alpha = 5;
  float beta = 6;
}
```

#### HybridQueryResponse
```protobuf
message HybridQueryResponse {
  repeated HybridResultProto results = 1;
}

message HybridResultProto {
  NodeProto node = 1;
  float score = 2;
}
```

---

### Client Examples

#### Python

```python
import grpc
from barq_graphdb.proto import barq_pb2, barq_pb2_grpc

channel = grpc.insecure_channel('localhost:50051')
stub = barq_pb2_grpc.BarqServiceStub(channel)

# Create node
node = barq_pb2.NodeProto(id=1, label="User")
response = stub.CreateNode(barq_pb2.CreateNodeRequest(node=node))

# Set embedding (binary protocol - 10x faster for large vectors)
stub.SetEmbedding(barq_pb2.SetEmbeddingRequest(
    id=1,
    embedding=[0.1, 0.2, 0.3, 0.4]
))

# Hybrid query
results = stub.HybridQuery(barq_pb2.HybridQueryRequest(
    start_node_id=1,
    query_embedding=[0.1, 0.2, 0.3, 0.4],
    max_hops=3,
    k=10,
    alpha=0.7,
    beta=0.3
))
for r in results.results:
    print(f"Node {r.node.id}: score={r.score}")
```

#### Go

```go
conn, _ := grpc.Dial("localhost:50051", grpc.WithInsecure())
client := pb.NewBarqServiceClient(conn)

// Create node
resp, _ := client.CreateNode(ctx, &pb.CreateNodeRequest{
    Node: &pb.NodeProto{Id: 1, Label: "User"},
})

// Set embedding
client.SetEmbedding(ctx, &pb.SetEmbeddingRequest{
    Id: 1,
    Embedding: []float32{0.1, 0.2, 0.3},
})
```

#### Node.js

```javascript
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

const packageDef = protoLoader.loadSync('proto/barq.proto');
const barq = grpc.loadPackageDefinition(packageDef).barq;

const client = new barq.BarqService(
    'localhost:50051',
    grpc.credentials.createInsecure()
);

client.CreateNode({
    node: { id: 1, label: 'User' }
}, (err, response) => {
    console.log(`Created: ${response.node.id}`);
});
```

---

## Error Handling

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 400 | Bad Request |
| 404 | Not Found |
| 500 | Internal Error |

### gRPC Status Codes

| Code | Meaning |
|------|---------|
| OK | Success |
| INVALID_ARGUMENT | Invalid parameters |
| NOT_FOUND | Resource not found |
| INTERNAL | Server error |

---

## Performance Tips

1. **Use gRPC for embeddings**: Binary protocol is ~10x faster for large vectors
2. **Batch operations**: Group creates before querying
3. **Tune alpha/beta**: Start with `alpha=0.7, beta=0.3` for semantic-first search
4. **Limit hops**: Keep `max_hops <= 4` for sub-ms latency

---

## See Also

- [Benchmark Results](./BENCHMARK_RESULTS.md)
- [Competitive Analysis](./COMPETITIVE_ANALYSIS.md)
- [Production Deployment](./PRODUCTION_DEPLOYMENT.md)
