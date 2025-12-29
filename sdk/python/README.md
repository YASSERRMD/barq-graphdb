# Barq-GraphDB Python SDK

Python client library for Barq-GraphDB - A production-grade graph+vector database for AI agents.

## Installation

```bash
pip install barq-graphdb
```

Or install from source:

```bash
cd sdk/python
pip install -e .
```

## Quick Start

```python
from barq_graphdb import BarqClient, Node, Edge, Decision, HybridParams

# Connect to the server
client = BarqClient("http://localhost:3000")

# Check health
print(client.health())

# Create nodes
client.create_node(Node(id=1, label="User"))
client.create_node(Node(id=2, label="Document"))

# Create edge
client.add_edge(from_node=1, to_node=2, edge_type="OWNS")

# Set embeddings
client.set_embedding(1, [0.1, 0.2, 0.3])
client.set_embedding(2, [0.2, 0.3, 0.4])

# Hybrid query
results = client.hybrid_query(
    start=1,
    query_embedding=[0.1, 0.2, 0.3],
    max_hops=3,
    k=5,
    params=HybridParams(alpha=0.7, beta=0.3)
)

for result in results:
    print(f"Node {result.id}: score={result.score:.3f}, path={result.path}")

# Record agent decisions
decision = Decision(
    agent_id=42,
    root_node=1,
    path=[1, 2],
    score=0.95,
    notes="Initial analysis"
)
recorded = client.record_decision(decision)
print(f"Decision ID: {recorded.id}")

# List decisions
decisions = client.list_decisions(agent_id=42)
for d in decisions:
    print(f"Decision {d.id}: path={d.path}, score={d.score}")
```

## API Reference

### BarqClient

Main client class for interacting with Barq-GraphDB.

#### Methods

- `health()` - Check server health
- `stats()` - Get database statistics
- `create_node(node)` - Create a new node
- `list_nodes()` - List all nodes
- `create_edge(edge)` - Create a new edge
- `add_edge(from_node, to_node, edge_type)` - Add an edge
- `set_embedding(node_id, embedding)` - Set node embedding
- `hybrid_query(...)` - Perform hybrid query
- `record_decision(decision)` - Record agent decision
- `list_decisions(agent_id)` - List agent decisions

### Models

- `Node` - Graph node with optional embedding
- `Edge` - Directed edge between nodes
- `HybridParams` - Parameters for hybrid scoring
- `HybridResult` - Result from hybrid query
- `Decision` - Agent decision record
- `Stats` - Database statistics

## License

MIT License
