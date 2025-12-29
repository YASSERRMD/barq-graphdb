#!/usr/bin/env python3
"""
Test script for Barq-GraphDB Python SDK.

This script tests all SDK functionality against a running Barq-GraphDB server.
"""

import sys
import os

# Add the SDK to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "barq_graphdb"))

from barq_graphdb import BarqClient, Node, Edge, Decision, HybridParams


def test_sdk():
    """Run all SDK tests."""
    print("=" * 60)
    print("Barq-GraphDB Python SDK Test Suite")
    print("=" * 60)
    
    # Connect to server
    client = BarqClient("http://localhost:3000")
    
    # Test 1: Health check
    print("\n[TEST 1] Health Check")
    health = client.health()
    assert health["status"] == "healthy", "Health check failed"
    print(f"  Status: {health['status']}")
    print(f"  Version: {health['version']}")
    print("  PASSED")
    
    # Test 2: Create nodes
    print("\n[TEST 2] Create Nodes")
    client.create_node(Node(id=1, label="Alice"))
    client.create_node(Node(id=2, label="Bob"))
    client.create_node(Node(id=3, label="Charlie"))
    client.create_node(Node(id=4, label="Document1"))
    client.create_node(Node(id=5, label="Document2"))
    print("  Created 5 nodes")
    print("  PASSED")
    
    # Test 3: List nodes
    print("\n[TEST 3] List Nodes")
    nodes = client.list_nodes()
    assert len(nodes) >= 5, f"Expected at least 5 nodes, got {len(nodes)}"
    print(f"  Found {len(nodes)} nodes:")
    for node in nodes[:5]:
        print(f"    - Node {node.id}: {node.label}")
    print("  PASSED")
    
    # Test 4: Create edges
    print("\n[TEST 4] Create Edges")
    client.add_edge(1, 2, "KNOWS")
    client.add_edge(1, 3, "KNOWS")
    client.add_edge(2, 4, "OWNS")
    client.add_edge(3, 5, "OWNS")
    client.add_edge(1, 4, "CREATED")
    print("  Created 5 edges")
    print("  PASSED")
    
    # Test 5: Set embeddings
    print("\n[TEST 5] Set Embeddings")
    client.set_embedding(1, [0.0, 0.0, 0.0])
    client.set_embedding(2, [1.0, 0.0, 0.0])
    client.set_embedding(3, [0.0, 1.0, 0.0])
    client.set_embedding(4, [1.0, 1.0, 0.0])
    client.set_embedding(5, [0.0, 1.0, 1.0])
    print("  Set embeddings for 5 nodes")
    print("  PASSED")
    
    # Test 6: Database stats
    print("\n[TEST 6] Database Stats")
    stats = client.stats()
    print(f"  Node count: {stats.node_count}")
    print(f"  Edge count: {stats.edge_count}")
    print(f"  Vector count: {stats.vector_count}")
    print(f"  Decision count: {stats.decision_count}")
    assert stats.node_count >= 5, "Expected at least 5 nodes"
    assert stats.edge_count >= 5, "Expected at least 5 edges"
    assert stats.vector_count >= 5, "Expected at least 5 vectors"
    print("  PASSED")
    
    # Test 7: Hybrid query
    print("\n[TEST 7] Hybrid Query")
    results = client.hybrid_query(
        start=1,
        query_embedding=[0.0, 0.0, 0.0],
        max_hops=3,
        k=5,
        params=HybridParams(alpha=0.7, beta=0.3)
    )
    print(f"  Found {len(results)} results:")
    for r in results:
        print(f"    - Node {r.id}: score={r.score:.3f}, dist={r.vector_distance:.3f}, hops={r.graph_distance}, path={r.path}")
    assert len(results) > 0, "Expected at least 1 result"
    assert results[0].id == 1, "Node 1 should be first (closest to query)"
    print("  PASSED")
    
    # Test 8: Record decision
    print("\n[TEST 8] Record Decision")
    decision = Decision(
        agent_id=100,
        root_node=1,
        path=[1, 2, 4],
        score=0.95,
        notes="Test decision from Python SDK"
    )
    recorded = client.record_decision(decision)
    print(f"  Decision ID: {recorded.id}")
    print(f"  Agent ID: {recorded.agent_id}")
    print(f"  Path: {recorded.path}")
    print(f"  Score: {recorded.score}")
    assert recorded.id is not None, "Decision should have an ID"
    print("  PASSED")
    
    # Test 9: List decisions
    print("\n[TEST 9] List Decisions")
    decisions = client.list_decisions(agent_id=100)
    print(f"  Found {len(decisions)} decisions for agent 100:")
    for d in decisions:
        print(f"    - Decision {d.id}: path={d.path}, score={d.score}")
    assert len(decisions) >= 1, "Expected at least 1 decision"
    print("  PASSED")
    
    # Test 10: Alpha-only hybrid query
    print("\n[TEST 10] Alpha-Only Hybrid Query")
    results = client.hybrid_query(
        start=1,
        query_embedding=[1.0, 1.0, 0.0],  # Closest to node 4
        max_hops=3,
        k=3,
        params=HybridParams(alpha=1.0, beta=0.0)  # Only vector matters
    )
    print(f"  Query: [1.0, 1.0, 0.0] with alpha=1.0, beta=0.0")
    print(f"  Top 3 results:")
    for r in results:
        print(f"    - Node {r.id}: score={r.score:.3f}, vector_dist={r.vector_distance:.3f}")
    # Node 4 has embedding [1, 1, 0] - should be closest
    assert results[0].id == 4, f"Expected node 4 to be first, got {results[0].id}"
    print("  PASSED")
    
    # Summary
    print("\n" + "=" * 60)
    print("ALL 10 TESTS PASSED")
    print("=" * 60)
    
    client.close()
    return True


if __name__ == "__main__":
    try:
        success = test_sdk()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\nTEST FAILED: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
