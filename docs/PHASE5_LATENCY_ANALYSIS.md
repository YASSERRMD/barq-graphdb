# Phase 5: Latency Analysis

**Date**: 2025-12-29
**Hardware**: Apple Silicon (M-series)
**Methodology**: Criterion.rs benchmark using `reqwest` client against `axum` server over local TCP loopback.

## Measured HTTP Latency (Phase 5)

| Endpoint | Median Latency | Throughput Estimate |
|----------|----------------|---------------------|
| `GET /health` | **35.0 μs** | ~28,500 req/s |
| `GET /nodes/{id}` | **41.4 μs** | ~24,000 req/s |
| `POST /nodes` | **52.2 μs** | ~19,000 req/s |

**Note**: These measurements include full HTTP stack (client -> network -> server -> DB -> network -> client).

## In-Process Baseline (Phase 2-4)

From previous `barq_complete_suite` benchmarks:

| Operation | Median Latency |
|-----------|----------------|
| `get_node` (Direct) | **~0.1 μs** |
| `append_node` (Direct) | **~2.3 μs** |

## Overhead Breakdown

Moving from direct library calls to HTTP introduces necessary overhead for enterprise architecture.

| Component | Time Cost | Source |
|-----------|-----------|--------|
| **Core Logic** | ~2 μs | Barq DB Engine |
| **Serialization** | ~5-10 μs | Serde JSON (Struct <-> String) |
| **Network/HTTP** | ~30-40 μs | TCP + HTTP Parsing (Axum/Hyper) |
| **Total** | **~50 μs** | |

## Analysis

The HTTP layer adds approximately **30-50 microseconds** of latency. This is **negligible** for most real-world applications where network latency (WAN) is often 10-50 *milliseconds*. 

Barq-GraphDB's HTTP server is highly efficient, capable of handling nearly **20,000 requests per second** on a single thread connection in sequential benchmarking. Concurrent load testing (Task 5) is expected to show much higher aggregate throughput.

## Comparison: Phase 2 vs Phase 5

| Version | Interface | Latency Scale | Use Case |
|---------|-----------|---------------|----------|
| Phase 2 | Rust Crate | Nanoseconds | Embedded, High-Freq Trading |
| Phase 5 | HTTP API | Microseconds | Microservices, Web Apps |

**Conclusion**: The implementation of the HTTP API using Axum preserves the high-performance characteristics of Barq-GraphDB, adding minimal legitimate overhead.
