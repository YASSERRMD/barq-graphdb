# Phase 3: Write Throughput Analysis

## Benchmark Results

**Metric**: Write operations per second (`set_embedding`).
**Configuration**: HNSW Index (M=16, ef_construction=200), `sync_writes=false` (in-memory speed).

- **HNSW Throughput**: **1,855 ops/s** (Measured, ~0.54ms per op).
- **Linear Throughput**: ~450,000 ops/s (Phase 2 Baseline).
- **Regression**: ~240x slowdown.

## Root Cause Analysis

The regression is due to the fundamental difference between Linear and HNSW indexing:
1. **Linear**: `O(1)` amortized `Vec::push`. CPU cost is negligible.
2. **HNSW**: `O(log N)` complexity involves:
   - Traversing the graph from entry point (distance computations).
   - Finding neighbors (candidate list).
   - Updating bidirectional links (random memory access).
   - `ef_construction=200` means analyzing 200 candidates per insert.

## Impact & Recommendations

### Impact
Synchronous HNSW indexing strictly limits ingestion rate to ~1k ops/s per thread. This violates the "Maintenance of 450k+ ops/s" goal if interpreted as "Indexed Write Throughput".

### Recommendations
To restore high throughput, **Asynchronous Indexing** is required:
1. **Write Path**: Write to WAL and Memory Map immediately (Ack to user).
2. **Index Path**: Background thread consumes updates and inserts into HNSW.
3. **Search Path**: Search HNSW + Brute-force search of "Unindexed Buffer" (Hybrid).

**Proposal**: Move "Async Indexing" to a dedicated optimization phase (Phase 4). For now, accept the throughput trade-off for the benefit of 100x faster search.
