# Barq-GraphDB API Reference

**Version**: v0.6.0  
**Base URL**: `http://localhost:3000` (default)

Barq-GraphDB provides a high-performance REST API for graph and vector operations. All request and response bodies are JSON.

## Latency SLA
- **Typical Latency**: 35-60 μs (local loopback)
- **SLA Guarantee**: < 5ms for single-node operations (99th percentile, excluding network)

---

## 1. System

### GET /health
Check API server health.

- **Latency**: ~35 μs
- **Status Codes**: 200 OK
- **Response**:
```json
{
  "status": "healthy",
  "version": "0.6.0"
}
```

### GET /stats
Get database statistics.

- **Status Codes**: 200 OK
- **Response**:
```json
{
  "node_count": 1000,
  "edge_count": 5000,
  "vector_count": 1000,
  "decision_count": 50
}
```

---

## 2. Nodes

### GET /nodes/{id}
Retrieve a single node.

- **Latency**: ~41 μs
- **Status Codes**: 
  - 200 OK
  - 404 Not Found
- **Response**:
```json
{
  "id": 1,
  "label": "Concept Node",
  "embedding": [0.1, 0.2, ...],
  "agent_id": 101,
  "rule_tags": ["important", "validated"],
  "edges": [2, 5],
  "timestamp": 1234567890
}
```

### POST /nodes
Create a new node.

- **Latency**: ~52 μs
- **Status Codes**: 201 Created, 400 Bad Request
- **Request**:
```json
{
  "id": 1,
  "label": "Concept Node",
  "embedding": [0.1, 0.2, ...], // Optional
  "agent_id": 101, // Optional
  "rule_tags": ["tag1"] // Optional
}
```
- **Response**:
```json
{
  "status": "ok",
  "node_id": 1
}
```

### GET /nodes
List all nodes (paginated implicitly by full dump currently).

- **Status Codes**: 200 OK
- **Response**:
```json
{
  "nodes": [...],
  "count": 100
}
```

---

## 3. Edges

### POST /edges
Create a directed edge between two nodes.

- **Status Codes**: 201 Created
- **Request**:
```json
{
  "from": 1,
  "to": 2,
  "edge_type": "connects_to"
}
```
- **Response**:
```json
{
  "status": "ok",
  "from": 1,
  "to": 2
}
```

---

## 4. Vectors

### POST /embeddings
Set or update the vector embedding for a node.

- **Request**:
```json
{
  "id": 1,
  "embedding": [0.1, 0.2, ...]
}
```
- **Response**: `{"status": "ok", "node_id": 1}`

---

## 5. Queries

### POST /query/hybrid
Perform a hybrid graph+vector search.

- **Request**:
```json
{
  "query_embedding": [0.1, ...],
  "start": 1,
  "max_hops": 2,
  "k": 10,
  "alpha": 0.5,
  "beta": 0.5
}
```
- **Response**:
```json
{
  "results": [
    {
      "id": 5,
      "score": 0.85,
      "vector_distance": 0.1,
      "graph_distance": 1,
      "path": [1, 5]
    }
  ]
}
```

---

## 6. Audit Logging

### POST /decisions
Record an agent decision for audit trail.

- **Request**:
```json
{
  "agent_id": 101,
  "root_node": 1,
  "path": [1, 2, 5],
  "score": 0.95,
  "notes": "Selected best path based on utility."
}
```

### GET /decisions
List decisions for a specific agent.

- **Query Params**: `?agent_id=101`
- **Response**:
```json
{
    "decisions": [...]
}
```
