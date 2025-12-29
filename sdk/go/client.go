// Package barqgraphdb provides a Go client for Barq-GraphDB REST API.
package barqgraphdb

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

// Client is the main client for interacting with Barq-GraphDB.
type Client struct {
	baseURL    string
	httpClient *http.Client
}

// NewClient creates a new Barq-GraphDB client.
func NewClient(baseURL string) *Client {
	return &Client{
		baseURL: baseURL,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// NewClientWithTimeout creates a new client with custom timeout.
func NewClientWithTimeout(baseURL string, timeout time.Duration) *Client {
	return &Client{
		baseURL: baseURL,
		httpClient: &http.Client{
			Timeout: timeout,
		},
	}
}

// Node represents a graph node.
type Node struct {
	ID           uint64    `json:"id"`
	Label        string    `json:"label"`
	Embedding    []float32 `json:"embedding,omitempty"`
	AgentID      *uint64   `json:"agent_id,omitempty"`
	RuleTags     []string  `json:"rule_tags,omitempty"`
	Timestamp    *uint64   `json:"timestamp,omitempty"`
	HasEmbedding bool      `json:"has_embedding,omitempty"`
}

// Edge represents a directed edge between nodes.
type Edge struct {
	From     uint64 `json:"from"`
	To       uint64 `json:"to"`
	EdgeType string `json:"edge_type"`
}

// HybridParams contains parameters for hybrid queries.
type HybridParams struct {
	Alpha float32 `json:"alpha"`
	Beta  float32 `json:"beta"`
}

// DefaultHybridParams returns default hybrid parameters.
func DefaultHybridParams() HybridParams {
	return HybridParams{Alpha: 0.5, Beta: 0.5}
}

// HybridResult represents a result from a hybrid query.
type HybridResult struct {
	ID             uint64   `json:"id"`
	Score          float32  `json:"score"`
	VectorDistance float32  `json:"vector_distance"`
	GraphDistance  int      `json:"graph_distance"`
	Path           []uint64 `json:"path"`
}

// Decision represents an agent decision record.
type Decision struct {
	ID        *uint64  `json:"id,omitempty"`
	AgentID   uint64   `json:"agent_id"`
	RootNode  uint64   `json:"root_node"`
	Path      []uint64 `json:"path"`
	Score     float32  `json:"score"`
	Notes     *string  `json:"notes,omitempty"`
	CreatedAt *uint64  `json:"created_at,omitempty"`
}

// Stats represents database statistics.
type Stats struct {
	NodeCount     int `json:"node_count"`
	EdgeCount     int `json:"edge_count"`
	VectorCount   int `json:"vector_count"`
	DecisionCount int `json:"decision_count"`
}

// HealthResponse represents the health check response.
type HealthResponse struct {
	Status  string `json:"status"`
	Version string `json:"version"`
}

// Error represents an API error.
type Error struct {
	Message    string `json:"error"`
	StatusCode int    `json:"code"`
}

func (e *Error) Error() string {
	return fmt.Sprintf("BarqError [%d]: %s", e.StatusCode, e.Message)
}

func (c *Client) doRequest(method, endpoint string, body interface{}, result interface{}) error {
	var reqBody io.Reader
	if body != nil {
		jsonBytes, err := json.Marshal(body)
		if err != nil {
			return fmt.Errorf("failed to marshal request: %w", err)
		}
		reqBody = bytes.NewReader(jsonBytes)
	}

	req, err := http.NewRequest(method, c.baseURL+endpoint, reqBody)
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Accept", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("failed to read response: %w", err)
	}

	if resp.StatusCode >= 400 {
		var apiErr Error
		if json.Unmarshal(respBody, &apiErr) == nil && apiErr.Message != "" {
			apiErr.StatusCode = resp.StatusCode
			return &apiErr
		}
		return &Error{Message: string(respBody), StatusCode: resp.StatusCode}
	}

	if result != nil {
		if err := json.Unmarshal(respBody, result); err != nil {
			return fmt.Errorf("failed to unmarshal response: %w", err)
		}
	}

	return nil
}

// Health checks the server health.
func (c *Client) Health() (*HealthResponse, error) {
	var result HealthResponse
	err := c.doRequest("GET", "/health", nil, &result)
	return &result, err
}

// Stats returns database statistics.
func (c *Client) Stats() (*Stats, error) {
	var result Stats
	err := c.doRequest("GET", "/stats", nil, &result)
	return &result, err
}

// CreateNode creates a new node.
func (c *Client) CreateNode(node *Node) error {
	return c.doRequest("POST", "/nodes", node, nil)
}

// ListNodes returns all nodes.
func (c *Client) ListNodes() ([]Node, error) {
	var result struct {
		Nodes []Node `json:"nodes"`
		Count int    `json:"count"`
	}
	err := c.doRequest("GET", "/nodes", nil, &result)
	return result.Nodes, err
}

// CreateEdge creates a new edge.
func (c *Client) CreateEdge(edge *Edge) error {
	return c.doRequest("POST", "/edges", edge, nil)
}

// AddEdge is a convenience method to add an edge.
func (c *Client) AddEdge(from, to uint64, edgeType string) error {
	return c.CreateEdge(&Edge{From: from, To: to, EdgeType: edgeType})
}

// SetEmbedding sets the embedding for a node.
func (c *Client) SetEmbedding(nodeID uint64, embedding []float32) error {
	payload := struct {
		ID        uint64    `json:"id"`
		Embedding []float32 `json:"embedding"`
	}{
		ID:        nodeID,
		Embedding: embedding,
	}
	return c.doRequest("POST", "/embeddings", payload, nil)
}

// HybridQueryRequest represents a hybrid query request.
type HybridQueryRequest struct {
	Start          uint64    `json:"start"`
	QueryEmbedding []float32 `json:"query_embedding"`
	MaxHops        int       `json:"max_hops"`
	K              int       `json:"k"`
	Alpha          float32   `json:"alpha"`
	Beta           float32   `json:"beta"`
}

// HybridQuery performs a hybrid query combining vector similarity and graph distance.
func (c *Client) HybridQuery(start uint64, queryEmbedding []float32, maxHops, k int, params HybridParams) ([]HybridResult, error) {
	req := HybridQueryRequest{
		Start:          start,
		QueryEmbedding: queryEmbedding,
		MaxHops:        maxHops,
		K:              k,
		Alpha:          params.Alpha,
		Beta:           params.Beta,
	}

	var result struct {
		Results []HybridResult `json:"results"`
	}
	err := c.doRequest("POST", "/query/hybrid", req, &result)
	return result.Results, err
}

// RecordDecision records an agent decision.
func (c *Client) RecordDecision(decision *Decision) (*Decision, error) {
	var result struct {
		Status   string   `json:"status"`
		Decision Decision `json:"decision"`
	}
	err := c.doRequest("POST", "/decisions", decision, &result)
	return &result.Decision, err
}

// ListDecisions returns all decisions for a specific agent.
func (c *Client) ListDecisions(agentID uint64) ([]Decision, error) {
	endpoint := fmt.Sprintf("/decisions?agent_id=%d", agentID)
	var result struct {
		Decisions []Decision `json:"decisions"`
	}
	err := c.doRequest("GET", endpoint, nil, &result)
	return result.Decisions, err
}

// Close closes the client (no-op for HTTP client).
func (c *Client) Close() {
	// No-op for HTTP client
}
