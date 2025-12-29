# Barq-GraphDB Benchmark Results

**Date**: 2025-12-29  
**Hardware**: Apple Silicon (M-series)  
**Barq Version**: v0.5.1  
**Rust Version**: 1.83.0  
**Benchmark Tool**: Criterion.rs v0.5

---

## Executive Summary

These are **actual benchmark results** from running Criterion.rs benchmarks on local hardware.

### Measured Performance

| Operation | Time | Throughput |
|-----------|------|------------|
| Write 1,000 nodes | 2.39ms | **418k ops/s** |
| Write 10,000 nodes | 21.77ms | **459k ops/s** |
| Write 50,000 nodes | 111ms | **450k ops/s** |
| Full agentic workflow (1k nodes) | 20.1ms | - |
| Record 10 decisions | 131μs | **76k ops/s** |
| Record 500 decisions | 1.27ms | **394k ops/s** |
| Node lookup (10k db) | 89ns | **11.2M ops/s** |
| WAL reload (10k nodes) | 6.8ms | - |

---

## 1. Write Throughput (Measured)

### Test Setup
- **Operation**: `append_node` (single-threaded)
- **Measurement**: Criterion.rs with 10 samples

### Actual Results

| Nodes | Time (median) | Throughput | Per-node |
|-------|--------------|------------|----------|
| 100 | 424μs | 236k ops/s | 4.24μs |
| 1,000 | 2.39ms | **418k ops/s** | 2.39μs |
| 10,000 | 21.77ms | **459k ops/s** | 2.18μs |
| 50,000 | 111ms | **450k ops/s** | 2.22μs |

**Key Finding**: Consistent ~450k writes/second at scale.

### With Embeddings (128-dim)

| Nodes | Time | Per-node |
|-------|------|----------|
| 100 | 1.51ms | 15.1μs |
| 1,000 | 13.6ms | 13.6μs |
| 5,000 | 65.4ms | 13.1μs |

**Key Finding**: ~75k nodes/second with 128-dim embeddings.

---

## 2. Persistence & Reload (Measured)

### WAL Reload Performance

| Nodes | Reload Time | Read Speed |
|-------|------------|------------|
| 1,000 | 878μs | ~1.14M nodes/s |
| 5,000 | 3.65ms | ~1.37M nodes/s |
| 10,000 | 6.79ms | **~1.47M nodes/s** |

**Key Finding**: WAL replay at 1.4M+ nodes/second.

---

## 3. Node Lookup (Measured)

| Database Size | Lookup Time |
|---------------|-------------|
| 10,000 nodes | **89ns** |

**Key Finding**: O(1) HashMap lookup at 11.2M lookups/second.

---

## 4. Agent Decision Recording (Measured)

| Decisions | Total Time | Per Decision |
|-----------|-----------|--------------|
| 10 | 131μs | 13.1μs |
| 100 | 434μs | 4.3μs |
| 500 | 1.27ms | 2.5μs |

**Key Finding**: ~400k decisions/second at scale.

---

## 5. End-to-End Agentic Workflow (Measured)

### Scenario: Complete Agent Learning Cycle
1. Create 1,000 nodes with 128-dim embeddings
2. Create ~3,000 edges (scale-free graph)
3. Perform 10 hybrid queries
4. Record 10 agent decisions

### Result

| Full Workflow | Time |
|---------------|------|
| 1,000 nodes + edges + queries + decisions | **20.1ms** |

**Key Finding**: Complete agentic workload in 20ms.

---

## 6. Computational Performance Summary

Based on actual measurements:

```
Operation                          Throughput
─────────────────────────────────────────────────────
Node writes (no embedding)         450,000 ops/s
Node writes (with embedding)        75,000 ops/s
Node lookups                     11,200,000 ops/s
Decision recording                 400,000 ops/s
WAL replay                       1,470,000 nodes/s
```

---

## 7. Competitor Comparison (Estimated)

**Note**: Competitor numbers below are **estimates based on published benchmarks and documentation**, not measurements from our test environment. Run `scripts/setup_competitors.sh` to benchmark competitors locally.

| Operation | Barq (Measured) | Neo4j (Est.) | SurrealDB (Est.) |
|-----------|----------------|--------------|------------------|
| Write throughput | **450k/s** | 1-5k/s | 5-10k/s |
| Node lookup | **89ns** | 1-10μs | 1-5μs |
| Graph traversal | TBD | Optimized | Basic |
| Vector search | TBD | Plugin | Native |

**To verify competitor performance**, run:
```bash
./scripts/setup_competitors.sh
# Then implement competitor benchmarks
```

---

## Methodology

### Tools
- **Criterion.rs v0.5** with HTML reports
- **Sample size**: 10 iterations per benchmark
- **Warm-up**: 3 seconds

### Reproducibility

Run benchmarks locally:
```bash
# All benchmarks
cargo bench --benches

# Specific benchmark suite
cargo bench --bench phase0_storage
cargo bench --bench barq_complete_suite

# View HTML reports
open target/criterion/report/index.html
```

### Environment
- Single-threaded execution
- Temp directories for isolation
- Fresh database per iteration

---

## Conclusion

**Measured advantages of Barq-GraphDB:**

1. **450k writes/second** (single-threaded, no batching)
2. **89ns node lookups** (O(1) HashMap)
3. **1.4M nodes/second** WAL replay
4. **20ms end-to-end** agentic workflow
5. **400k decisions/second** audit trail recording

These numbers are from actual Criterion.rs runs, not estimates.

---

*Last updated: 2025-12-29 from actual benchmark runs*
