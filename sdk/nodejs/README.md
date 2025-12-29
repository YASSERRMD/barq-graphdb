# Barq-GraphDB Node.js SDK

Node.js/TypeScript client library for Barq-GraphDB - A production-grade graph+vector database for AI agents.

## Installation

```bash
npm install barq-graphdb
```

## Quick Start

```javascript
const { BarqClient } = require('barq-graphdb');
// Or with ES modules:
// import { BarqClient } from 'barq-graphdb';

async function main() {
  // Connect to server
  const client = new BarqClient('http://localhost:3000');

  // Check health
  const health = await client.health();
  console.log(`Server: ${health.status} v${health.version}`);

  // Create nodes
  await client.createNode({ id: 1, label: 'User' });
  await client.createNode({ id: 2, label: 'Document' });

  // Create edge
  await client.addEdge(1, 2, 'OWNS');

  // Set embeddings
  await client.setEmbedding(1, [0.1, 0.2, 0.3]);
  await client.setEmbedding(2, [0.2, 0.3, 0.4]);

  // Hybrid query
  const results = await client.hybridQuery(
    1,                    // start
    [0.1, 0.2, 0.3],      // query embedding
    3,                    // max hops
    5,                    // top k
    { alpha: 0.7, beta: 0.3 }
  );

  for (const r of results) {
    console.log(`Node ${r.id}: score=${r.score.toFixed(3)}, path=[${r.path}]`);
  }

  // Record decision
  const decision = await client.recordDecision({
    agent_id: 42,
    root_node: 1,
    path: [1, 2],
    score: 0.95,
    notes: 'Initial analysis'
  });
  console.log(`Decision ID: ${decision.id}`);

  // List decisions
  const decisions = await client.listDecisions(42);
  console.log(`Agent 42 has ${decisions.length} decisions`);
}

main().catch(console.error);
```

## TypeScript Support

This SDK is written in TypeScript and includes full type definitions.

```typescript
import { BarqClient, Node, HybridResult, Decision } from 'barq-graphdb';

const client = new BarqClient('http://localhost:3000');

const node: Node = {
  id: 1,
  label: 'User',
  embedding: [0.1, 0.2, 0.3]
};

await client.createNode(node);
```

## API Reference

### BarqClient Methods

- `health()` - Check server health
- `stats()` - Get database statistics
- `createNode(node)` - Create a node
- `listNodes()` - List all nodes
- `createEdge(edge)` - Create an edge
- `addEdge(from, to, edgeType)` - Add an edge
- `setEmbedding(nodeId, embedding)` - Set node embedding
- `hybridQuery(start, queryEmbedding, maxHops?, k?, params?)` - Perform hybrid query
- `recordDecision(decision)` - Record agent decision
- `listDecisions(agentId)` - List agent decisions

### Types

- `Node` - Graph node
- `Edge` - Directed edge
- `HybridParams` - Hybrid query parameters
- `HybridResult` - Hybrid query result
- `Decision` - Agent decision record
- `Stats` - Database statistics

## Requirements

- Node.js 18.0.0 or later (uses native fetch)

## License

MIT License
