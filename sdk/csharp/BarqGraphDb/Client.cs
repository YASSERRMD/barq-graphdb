using System.Net.Http.Json;
using System.Text.Json;

namespace BarqGraphDb;

/// <summary>
/// Exception thrown for Barq-GraphDB API errors.
/// </summary>
public class BarqException : Exception
{
    public int StatusCode { get; }

    public BarqException(string message, int statusCode = 0) : base(message)
    {
        StatusCode = statusCode;
    }
}

/// <summary>
/// Client for Barq-GraphDB REST API.
/// </summary>
public class BarqClient : IDisposable
{
    private readonly HttpClient _httpClient;
    private readonly JsonSerializerOptions _jsonOptions;
    private bool _disposed;

    /// <summary>
    /// Creates a new Barq-GraphDB client.
    /// </summary>
    /// <param name="baseUrl">Base URL of the Barq-GraphDB server.</param>
    /// <param name="timeout">Request timeout.</param>
    public BarqClient(string baseUrl, TimeSpan? timeout = null)
    {
        _httpClient = new HttpClient
        {
            BaseAddress = new Uri(baseUrl.TrimEnd('/')),
            Timeout = timeout ?? TimeSpan.FromSeconds(30)
        };
        _httpClient.DefaultRequestHeaders.Add("Accept", "application/json");

        _jsonOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
        };
    }

    private async Task<T> GetAsync<T>(string endpoint, CancellationToken ct = default)
    {
        var response = await _httpClient.GetAsync(endpoint, ct);
        await EnsureSuccessAsync(response);
        return await response.Content.ReadFromJsonAsync<T>(_jsonOptions, ct)
            ?? throw new BarqException("Empty response");
    }

    private async Task<T> PostAsync<T>(string endpoint, object? body, CancellationToken ct = default)
    {
        var response = await _httpClient.PostAsJsonAsync(endpoint, body, _jsonOptions, ct);
        await EnsureSuccessAsync(response);
        return await response.Content.ReadFromJsonAsync<T>(_jsonOptions, ct)
            ?? throw new BarqException("Empty response");
    }

    private async Task PostAsync(string endpoint, object? body, CancellationToken ct = default)
    {
        var response = await _httpClient.PostAsJsonAsync(endpoint, body, _jsonOptions, ct);
        await EnsureSuccessAsync(response);
    }

    private static async Task EnsureSuccessAsync(HttpResponseMessage response)
    {
        if (!response.IsSuccessStatusCode)
        {
            var content = await response.Content.ReadAsStringAsync();
            throw new BarqException(content, (int)response.StatusCode);
        }
    }

    /// <summary>
    /// Checks server health.
    /// </summary>
    public async Task<HealthResponse> HealthAsync(CancellationToken ct = default)
    {
        return await GetAsync<HealthResponse>("/health", ct);
    }

    /// <summary>
    /// Gets database statistics.
    /// </summary>
    public async Task<Stats> StatsAsync(CancellationToken ct = default)
    {
        return await GetAsync<Stats>("/stats", ct);
    }

    /// <summary>
    /// Creates a new node.
    /// </summary>
    public async Task CreateNodeAsync(Node node, CancellationToken ct = default)
    {
        await PostAsync("/nodes", node, ct);
    }

    /// <summary>
    /// Lists all nodes.
    /// </summary>
    public async Task<List<Node>> ListNodesAsync(CancellationToken ct = default)
    {
        var response = await GetAsync<NodesResponse>("/nodes", ct);
        return response.Nodes;
    }

    /// <summary>
    /// Creates a new edge.
    /// </summary>
    public async Task CreateEdgeAsync(Edge edge, CancellationToken ct = default)
    {
        await PostAsync("/edges", edge, ct);
    }

    /// <summary>
    /// Adds an edge between nodes.
    /// </summary>
    public async Task AddEdgeAsync(ulong from, ulong to, string edgeType, CancellationToken ct = default)
    {
        await CreateEdgeAsync(new Edge { From = from, To = to, EdgeType = edgeType }, ct);
    }

    /// <summary>
    /// Sets the embedding for a node.
    /// </summary>
    public async Task SetEmbeddingAsync(ulong nodeId, float[] embedding, CancellationToken ct = default)
    {
        await PostAsync("/embeddings", new SetEmbeddingRequest { Id = nodeId, Embedding = embedding }, ct);
    }

    /// <summary>
    /// Performs a hybrid query combining vector similarity and graph distance.
    /// </summary>
    public async Task<List<HybridResult>> HybridQueryAsync(
        ulong start,
        float[] queryEmbedding,
        int maxHops = 3,
        int k = 10,
        HybridParams? hybridParams = null,
        CancellationToken ct = default)
    {
        hybridParams ??= HybridParams.Default;
        var request = new HybridQueryRequest
        {
            Start = start,
            QueryEmbedding = queryEmbedding,
            MaxHops = maxHops,
            K = k,
            Alpha = hybridParams.Alpha,
            Beta = hybridParams.Beta
        };
        var response = await PostAsync<HybridQueryResponse>("/query/hybrid", request, ct);
        return response.Results;
    }

    /// <summary>
    /// Records an agent decision.
    /// </summary>
    public async Task<Decision> RecordDecisionAsync(Decision decision, CancellationToken ct = default)
    {
        var response = await PostAsync<DecisionResponse>("/decisions", decision, ct);
        return response.Decision;
    }

    /// <summary>
    /// Lists all decisions for a specific agent.
    /// </summary>
    public async Task<List<Decision>> ListDecisionsAsync(ulong agentId, CancellationToken ct = default)
    {
        var response = await GetAsync<DecisionsResponse>($"/decisions?agent_id={agentId}", ct);
        return response.Decisions;
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            _httpClient.Dispose();
            _disposed = true;
        }
        GC.SuppressFinalize(this);
    }
}
