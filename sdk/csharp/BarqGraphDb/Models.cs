using System.Text.Json.Serialization;

namespace BarqGraphDb;

/// <summary>
/// Represents a node in the graph.
/// </summary>
public class Node
{
    [JsonPropertyName("id")]
    public ulong Id { get; set; }

    [JsonPropertyName("label")]
    public string Label { get; set; } = string.Empty;

    [JsonPropertyName("embedding")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public float[]? Embedding { get; set; }

    [JsonPropertyName("agent_id")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public ulong? AgentId { get; set; }

    [JsonPropertyName("rule_tags")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<string>? RuleTags { get; set; }

    [JsonPropertyName("timestamp")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public ulong? Timestamp { get; set; }

    [JsonPropertyName("has_embedding")]
    public bool HasEmbedding { get; set; }
}

/// <summary>
/// Represents a directed edge between nodes.
/// </summary>
public class Edge
{
    [JsonPropertyName("from")]
    public ulong From { get; set; }

    [JsonPropertyName("to")]
    public ulong To { get; set; }

    [JsonPropertyName("edge_type")]
    public string EdgeType { get; set; } = string.Empty;
}

/// <summary>
/// Parameters for hybrid queries.
/// </summary>
public class HybridParams
{
    [JsonPropertyName("alpha")]
    public float Alpha { get; set; } = 0.5f;

    [JsonPropertyName("beta")]
    public float Beta { get; set; } = 0.5f;

    public static HybridParams Default => new() { Alpha = 0.5f, Beta = 0.5f };
}

/// <summary>
/// Result from a hybrid query.
/// </summary>
public class HybridResult
{
    [JsonPropertyName("id")]
    public ulong Id { get; set; }

    [JsonPropertyName("score")]
    public float Score { get; set; }

    [JsonPropertyName("vector_distance")]
    public float VectorDistance { get; set; }

    [JsonPropertyName("graph_distance")]
    public int GraphDistance { get; set; }

    [JsonPropertyName("path")]
    public List<ulong> Path { get; set; } = new();
}

/// <summary>
/// Represents an agent decision record.
/// </summary>
public class Decision
{
    [JsonPropertyName("id")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public ulong? Id { get; set; }

    [JsonPropertyName("agent_id")]
    public ulong AgentId { get; set; }

    [JsonPropertyName("root_node")]
    public ulong RootNode { get; set; }

    [JsonPropertyName("path")]
    public List<ulong> Path { get; set; } = new();

    [JsonPropertyName("score")]
    public float Score { get; set; }

    [JsonPropertyName("notes")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Notes { get; set; }

    [JsonPropertyName("created_at")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public ulong? CreatedAt { get; set; }
}

/// <summary>
/// Database statistics.
/// </summary>
public class Stats
{
    [JsonPropertyName("node_count")]
    public int NodeCount { get; set; }

    [JsonPropertyName("edge_count")]
    public int EdgeCount { get; set; }

    [JsonPropertyName("vector_count")]
    public int VectorCount { get; set; }

    [JsonPropertyName("decision_count")]
    public int DecisionCount { get; set; }
}

/// <summary>
/// Health check response.
/// </summary>
public class HealthResponse
{
    [JsonPropertyName("status")]
    public string Status { get; set; } = string.Empty;

    [JsonPropertyName("version")]
    public string Version { get; set; } = string.Empty;
}

// Internal request/response types

internal class HybridQueryRequest
{
    [JsonPropertyName("start")]
    public ulong Start { get; set; }

    [JsonPropertyName("query_embedding")]
    public float[] QueryEmbedding { get; set; } = Array.Empty<float>();

    [JsonPropertyName("max_hops")]
    public int MaxHops { get; set; }

    [JsonPropertyName("k")]
    public int K { get; set; }

    [JsonPropertyName("alpha")]
    public float Alpha { get; set; }

    [JsonPropertyName("beta")]
    public float Beta { get; set; }
}

internal class SetEmbeddingRequest
{
    [JsonPropertyName("id")]
    public ulong Id { get; set; }

    [JsonPropertyName("embedding")]
    public float[] Embedding { get; set; } = Array.Empty<float>();
}

internal class NodesResponse
{
    [JsonPropertyName("nodes")]
    public List<Node> Nodes { get; set; } = new();

    [JsonPropertyName("count")]
    public int Count { get; set; }
}

internal class HybridQueryResponse
{
    [JsonPropertyName("results")]
    public List<HybridResult> Results { get; set; } = new();
}

internal class DecisionResponse
{
    [JsonPropertyName("status")]
    public string Status { get; set; } = string.Empty;

    [JsonPropertyName("decision")]
    public Decision Decision { get; set; } = new();
}

internal class DecisionsResponse
{
    [JsonPropertyName("decisions")]
    public List<Decision> Decisions { get; set; } = new();
}
