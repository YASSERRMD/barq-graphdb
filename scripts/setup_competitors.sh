#!/bin/bash
# Setup script for benchmark competitor databases

set -e

echo "============================================"
echo "Barq-GraphDB Benchmark Setup"
echo "============================================"

echo ""
echo "[1/4] Starting competitor databases..."
docker compose -f docker-compose.benchmark.yml up -d neo4j surrealdb pgvector

echo ""
echo "[2/4] Waiting for services to be healthy..."
sleep 15

echo ""
echo "[3/4] Verifying Neo4j..."
until docker exec benchmark-neo4j cypher-shell -u neo4j -p password123 "RETURN 1" > /dev/null 2>&1; do
    echo "  Waiting for Neo4j..."
    sleep 5
done
echo "  Neo4j is ready!"

# Create test constraint
docker exec benchmark-neo4j cypher-shell -u neo4j -p password123 \
    "CREATE CONSTRAINT unique_node_id IF NOT EXISTS FOR (n:Node) REQUIRE n.id IS UNIQUE" || true

echo ""
echo "[4/4] Verifying SurrealDB..."
until curl -s http://localhost:8000/health > /dev/null 2>&1; do
    echo "  Waiting for SurrealDB..."
    sleep 5
done
echo "  SurrealDB is ready!"

echo ""
echo "============================================"
echo "All competitor databases are ready!"
echo "============================================"
echo ""
echo "Connection details:"
echo "  Neo4j:     bolt://localhost:7687 (neo4j/password123)"
echo "  SurrealDB: http://localhost:8000 (root/root)"
echo "  pgvector:  postgresql://benchmark:password123@localhost:5432/benchmark"
echo ""
echo "Run benchmarks with: cargo bench --benches"
