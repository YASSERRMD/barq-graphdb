# Barq-GraphDB Go SDK

Go client library for Barq-GraphDB - A production-grade graph+vector database for AI agents.

## Installation

```bash
go get github.com/YASSERRMD/barq-graphdb/sdk/go
```

## Quick Start

```go
package main

import (
    "fmt"
    "log"
    
    barq "github.com/YASSERRMD/barq-graphdb/sdk/go"
)

func main() {
    // Connect to server
    client := barq.NewClient("http://localhost:3000")
    defer client.Close()

    // Check health
    health, err := client.Health()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Server: %s v%s\n", health.Status, health.Version)

    // Create nodes
    client.CreateNode(&barq.Node{ID: 1, Label: "User"})
    client.CreateNode(&barq.Node{ID: 2, Label: "Document"})

    // Create edge
    client.AddEdge(1, 2, "OWNS")

    // Set embeddings
    client.SetEmbedding(1, []float32{0.1, 0.2, 0.3})
    client.SetEmbedding(2, []float32{0.2, 0.3, 0.4})

    // Hybrid query
    results, err := client.HybridQuery(
        1,                              // start node
        []float32{0.1, 0.2, 0.3},       // query embedding
        3,                              // max hops
        5,                              // top k
        barq.HybridParams{Alpha: 0.7, Beta: 0.3},
    )
    if err != nil {
        log.Fatal(err)
    }

    for _, r := range results {
        fmt.Printf("Node %d: score=%.3f, path=%v\n", r.ID, r.Score, r.Path)
    }

    // Record decision
    notes := "Analysis complete"
    decision, _ := client.RecordDecision(&barq.Decision{
        AgentID:  42,
        RootNode: 1,
        Path:     []uint64{1, 2},
        Score:    0.95,
        Notes:    &notes,
    })
    fmt.Printf("Decision ID: %d\n", *decision.ID)

    // List decisions
    decisions, _ := client.ListDecisions(42)
    fmt.Printf("Agent 42 has %d decisions\n", len(decisions))
}
```

## API Reference

### Client Methods

- `NewClient(baseURL)` - Create new client
- `Health()` - Check server health
- `Stats()` - Get database statistics
- `CreateNode(node)` - Create a node
- `ListNodes()` - List all nodes
- `CreateEdge(edge)` - Create an edge
- `AddEdge(from, to, edgeType)` - Add an edge
- `SetEmbedding(nodeID, embedding)` - Set node embedding
- `HybridQuery(...)` - Perform hybrid query
- `RecordDecision(decision)` - Record agent decision
- `ListDecisions(agentID)` - List agent decisions

### Types

- `Node` - Graph node
- `Edge` - Directed edge
- `HybridParams` - Hybrid query parameters
- `HybridResult` - Hybrid query result
- `Decision` - Agent decision record
- `Stats` - Database statistics

## License

MIT License
