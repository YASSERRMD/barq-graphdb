//! Phase 3 HNSW Benchmarks
//!
//! Compares Linear vs HNSW vector search performance.

use barq_graphdb::bench_utils::{generate_random_nodes, generate_random_query};
use barq_graphdb::storage::{BarqGraphDb, DbOptions, IndexType};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::TempDir;

fn benchmark_hnsw_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_hnsw_comparison");
    group.sample_size(10); // Adjust based on runtime

    // Testing sizes: 2k, 5k, 10k, 50k
    // Note: 50k might be slow for Linear, so reduce iterations if needed
    let sizes = [2000, 5000, 10000, 50000];

    for &size in &sizes {
        // Benchmark Linear Search
        group.bench_with_input(BenchmarkId::new("Linear", size), &size, |b, &s| {
            let dir = TempDir::new().unwrap();
            let mut opts = DbOptions::new(dir.path().to_path_buf());
            opts.index_type = IndexType::Linear;

            let mut db = BarqGraphDb::open(opts).unwrap();
            let nodes_data = generate_random_nodes(s, 128);
            for node in nodes_data {
                let id = node.id;
                let emb = node.embedding.clone();
                db.append_node(node).unwrap();
                db.set_embedding(id, emb).unwrap();
            }

            b.iter(|| {
                let query = generate_random_query(128);
                let _results = db.knn_search(&query, 10);
            });
        });

        // Benchmark HNSW Search
        group.bench_with_input(BenchmarkId::new("HNSW", size), &size, |b, &s| {
            let dir = TempDir::new().unwrap();
            let mut opts = DbOptions::new(dir.path().to_path_buf());
            opts.index_type = IndexType::Hnsw;

            let mut db = BarqGraphDb::open(opts).unwrap();
            let nodes_data = generate_random_nodes(s, 128);
            for node in nodes_data {
                let id = node.id;
                let emb = node.embedding.clone();
                db.append_node(node).unwrap();
                db.set_embedding(id, emb).unwrap();
            }

            b.iter(|| {
                let query = generate_random_query(128);
                let _results = db.knn_search(&query, 10);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_hnsw_comparison);
criterion_main!(benches);
