package barqgraphdb

import (
	"fmt"
	"testing"
)

func TestClient(t *testing.T) {
	client := NewClient("http://localhost:3000")
	defer client.Close()

	// Test health
	health, err := client.Health()
	if err != nil {
		t.Fatalf("Health check failed: %v", err)
	}
	if health.Status != "healthy" {
		t.Errorf("Expected healthy status, got %s", health.Status)
	}
	fmt.Printf("Health: %+v\n", health)

	// Test create nodes
	for i := uint64(100); i < 105; i++ {
		err := client.CreateNode(&Node{
			ID:    i,
			Label: fmt.Sprintf("TestNode%d", i),
		})
		if err != nil {
			t.Fatalf("CreateNode failed: %v", err)
		}
	}
	fmt.Println("Created 5 nodes")

	// Test list nodes
	nodes, err := client.ListNodes()
	if err != nil {
		t.Fatalf("ListNodes failed: %v", err)
	}
	if len(nodes) < 5 {
		t.Errorf("Expected at least 5 nodes, got %d", len(nodes))
	}
	fmt.Printf("Found %d nodes\n", len(nodes))

	// Test add edge
	err = client.AddEdge(100, 101, "CONNECTS")
	if err != nil {
		t.Fatalf("AddEdge failed: %v", err)
	}
	fmt.Println("Created edge")

	// Test set embedding
	err = client.SetEmbedding(100, []float32{0.1, 0.2, 0.3})
	if err != nil {
		t.Fatalf("SetEmbedding failed: %v", err)
	}
	err = client.SetEmbedding(101, []float32{0.2, 0.3, 0.4})
	if err != nil {
		t.Fatalf("SetEmbedding failed: %v", err)
	}
	fmt.Println("Set embeddings")

	// Test stats
	stats, err := client.Stats()
	if err != nil {
		t.Fatalf("Stats failed: %v", err)
	}
	fmt.Printf("Stats: %+v\n", stats)

	// Test hybrid query
	results, err := client.HybridQuery(100, []float32{0.1, 0.2, 0.3}, 3, 5, DefaultHybridParams())
	if err != nil {
		t.Fatalf("HybridQuery failed: %v", err)
	}
	fmt.Printf("Hybrid results: %d\n", len(results))
	for _, r := range results {
		fmt.Printf("  Node %d: score=%.3f, path=%v\n", r.ID, r.Score, r.Path)
	}

	// Test record decision
	notes := "Test from Go SDK"
	decision, err := client.RecordDecision(&Decision{
		AgentID:  200,
		RootNode: 100,
		Path:     []uint64{100, 101},
		Score:    0.9,
		Notes:    &notes,
	})
	if err != nil {
		t.Fatalf("RecordDecision failed: %v", err)
	}
	fmt.Printf("Decision: %+v\n", decision)

	// Test list decisions
	decisions, err := client.ListDecisions(200)
	if err != nil {
		t.Fatalf("ListDecisions failed: %v", err)
	}
	if len(decisions) == 0 {
		t.Error("Expected at least 1 decision")
	}
	fmt.Printf("Found %d decisions for agent 200\n", len(decisions))

	fmt.Println("\nAll tests passed!")
}
