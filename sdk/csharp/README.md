# Barq-GraphDB C# SDK

C# client library for Barq-GraphDB - A production-grade graph+vector database for AI agents.

## Installation

```bash
dotnet add package BarqGraphDb
```

Or add to your `.csproj`:

```xml
<PackageReference Include="BarqGraphDb" Version="0.1.0" />
```

## Quick Start

```csharp
using BarqGraphDb;

// Connect to server
using var client = new BarqClient("http://localhost:3000");

// Check health
var health = await client.HealthAsync();
Console.WriteLine($"Server: {health.Status} v{health.Version}");

// Create nodes
await client.CreateNodeAsync(new Node { Id = 1, Label = "User" });
await client.CreateNodeAsync(new Node { Id = 2, Label = "Document" });

// Create edge
await client.AddEdgeAsync(from: 1, to: 2, edgeType: "OWNS");

// Set embeddings
await client.SetEmbeddingAsync(1, new[] { 0.1f, 0.2f, 0.3f });
await client.SetEmbeddingAsync(2, new[] { 0.2f, 0.3f, 0.4f });

// Hybrid query
var results = await client.HybridQueryAsync(
    start: 1,
    queryEmbedding: new[] { 0.1f, 0.2f, 0.3f },
    maxHops: 3,
    k: 5,
    hybridParams: new HybridParams { Alpha = 0.7f, Beta = 0.3f }
);

foreach (var r in results)
{
    Console.WriteLine($"Node {r.Id}: score={r.Score:F3}, path=[{string.Join(",", r.Path)}]");
}

// Record decision
var decision = await client.RecordDecisionAsync(new Decision
{
    AgentId = 42,
    RootNode = 1,
    Path = new List<ulong> { 1, 2 },
    Score = 0.95f,
    Notes = "Initial analysis"
});
Console.WriteLine($"Decision ID: {decision.Id}");

// List decisions
var decisions = await client.ListDecisionsAsync(agentId: 42);
Console.WriteLine($"Agent 42 has {decisions.Count} decisions");
```

## API Reference

### BarqClient Methods

All methods are async and support cancellation tokens.

- `HealthAsync()` - Check server health
- `StatsAsync()` - Get database statistics
- `CreateNodeAsync(node)` - Create a node
- `ListNodesAsync()` - List all nodes
- `CreateEdgeAsync(edge)` - Create an edge
- `AddEdgeAsync(from, to, edgeType)` - Add an edge
- `SetEmbeddingAsync(nodeId, embedding)` - Set node embedding
- `HybridQueryAsync(...)` - Perform hybrid query
- `RecordDecisionAsync(decision)` - Record agent decision
- `ListDecisionsAsync(agentId)` - List agent decisions

### Models

- `Node` - Graph node
- `Edge` - Directed edge
- `HybridParams` - Hybrid query parameters
- `HybridResult` - Hybrid query result
- `Decision` - Agent decision record
- `Stats` - Database statistics

## Requirements

- .NET 8.0 or later
- System.Text.Json

## License

MIT License
