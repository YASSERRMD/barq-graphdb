# Phase 4 Release Notes: Async Indexing & Throughput Optimization

## Overview
Phase 4 focused on resolving the write throughput regression introduced by the synchronous HNSW indexing in Phase 3. By decoupling the critical write path from the heavy vector indexing operations, we restoration high ingestion rates suitable for production environments.

## Features

### 1. Asynchronous Vector Indexing
- **Decoupled Architecture**: `append_node` and `set_embedding` now delegate HNSW insertion to a background worker thread via a bounded channel.
- **Configurable**: Enabled via `DbOptions.async_indexing: true`. Defaults to `false` (Sync) for strict consistency testing.
- **Result**: The write path now only waits for WAL IO and JSON serialization, removing the `O(log N)` graph construction latency from the user response time.

### 2. Thread-Safe Vector Indices
- **Concurrency**: Refactored `VectorIndex` trait and implementations to support fully concurrent access (Send + Sync).
- **HNSW**: Switched internal mappings to `DashMap` and atomic counters, allowing lock-free reads and fine-grained locking writes.
- **Linear**: Wrapped in `RwLock` for safety.

## Performance Benchmark

- **Metric**: Write Throughput (Ops/Sec) with persisting to WAL (JSON).
- **Phase 3 (Sync HNSW)**: ~1,855 ops/s.
- **Phase 4 (Async HNSW)**: ~27,800 ops/s.
- **Improvement**: **~15x speedup**.

## Usage

```rust
let mut opts = DbOptions::new(path);
opts.index_type = IndexType::Hnsw;
opts.async_indexing = true; // Enable high throughput
let db = BarqGraphDb::open(opts)?;
```

## Next Steps
- Implement binary serialization (e.g., Bincode) to break the 30k ops/s JSON limit and reach >100k ops/s.
