//! Phase 3 Write Throughput Benchmarks
//!
//! Verifies write performance with HNSW index enabled.
//! Metrics: Ops/sec for append_node and set_embedding.

use barq_graphdb::bench_utils::{generate_random_nodes, generate_random_query};
use barq_graphdb::storage::{BarqGraphDb, DbOptions, IndexType};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tempfile::TempDir;

fn benchmark_write_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_write");
    group.sample_size(10);
    
    // Test with batch of 10k ops
    let size = 10_000;
    
    group.throughput(Throughput::Elements(size as u64));

    // Benchmark set_embedding with HNSW (Batch)
    group.bench_with_input(BenchmarkId::new("set_embedding_hnsw_nosync", size), &size, |b, &s| {
        b.iter_batched(
            || {
                let dir = TempDir::new().unwrap();
                let mut opts = DbOptions::new(dir.path().to_path_buf());
                opts.index_type = IndexType::Hnsw;
                opts.sync_writes = false; // Bypass disk
                opts.async_indexing = true; // Bypass sync HNSW insert
                
                let mut db = BarqGraphDb::open(opts).unwrap();
                let nodes_data = generate_random_nodes(s, 128);
                for node in &nodes_data {
                    db.append_node(node.clone()).unwrap();
                }
                (dir, db, nodes_data)
            },
            |(_dir, mut db, nodes)| {
                for node in nodes {
                    // This includes HNSW insert
                    db.set_embedding(node.id, node.embedding).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, benchmark_write_ops);
criterion_main!(benches);
