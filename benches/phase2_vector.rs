//! Phase 2 Vector Benchmarks
//!
//! Benchmarks for vector operations: kNN search, embedding storage.

use barq_graphdb::bench_utils::{generate_random_nodes, generate_random_query};
use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::TempDir;

/// Benchmark kNN search with different dataset sizes.
fn benchmark_knn_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("knn_search");
    group.sample_size(10);

    for nodes in [100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(nodes), nodes, |b, &n| {
            let dir = TempDir::new().unwrap();
            let mut db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();

            let nodes_data = generate_random_nodes(n, 128);
            for node in nodes_data {
                let embedding = node.embedding.clone();
                let id = node.id;
                db.append_node(node).unwrap();
                db.set_embedding(id, embedding).unwrap();
            }

            b.iter(|| {
                let query = generate_random_query(128);
                let results = db.knn_search(&query, 10);
                criterion::black_box(results);
            });
        });
    }
    group.finish();
}

/// Benchmark kNN with varying k values.
fn benchmark_knn_varying_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("knn_varying_k");
    group.sample_size(10);

    // Setup 5000 node database
    let dir = TempDir::new().unwrap();
    let mut db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();

    let nodes_data = generate_random_nodes(5000, 128);
    for node in nodes_data {
        let embedding = node.embedding.clone();
        let id = node.id;
        db.append_node(node).unwrap();
        db.set_embedding(id, embedding).unwrap();
    }

    for k in [1, 5, 10, 25, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(k), k, |b, &k_val| {
            b.iter(|| {
                let query = generate_random_query(128);
                let results = db.knn_search(&query, k_val);
                criterion::black_box(results);
            });
        });
    }
    group.finish();
}

/// Benchmark embedding dimension impact.
fn benchmark_embedding_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("embedding_dimensions");
    group.sample_size(10);

    for dim in [64, 128, 256, 512].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, &d| {
            let dir = TempDir::new().unwrap();
            let mut db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();

            let nodes_data = generate_random_nodes(1000, d);
            for node in nodes_data {
                let embedding = node.embedding.clone();
                let id = node.id;
                db.append_node(node).unwrap();
                db.set_embedding(id, embedding).unwrap();
            }

            b.iter(|| {
                let query = generate_random_query(d);
                let results = db.knn_search(&query, 10);
                criterion::black_box(results);
            });
        });
    }
    group.finish();
}

/// Benchmark embedding set operation.
fn benchmark_set_embedding(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_embedding");
    group.sample_size(10);

    for nodes in [100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(nodes), nodes, |b, &n| {
            b.iter_batched(
                || {
                    let dir = TempDir::new().unwrap();
                    let mut db =
                        BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                    let nodes_data = generate_random_nodes(n, 0);
                    for node in nodes_data {
                        db.append_node(node).unwrap();
                    }
                    let embeddings: Vec<_> = (0..n)
                        .map(|i| (i as u64, generate_random_query(128)))
                        .collect();
                    (dir, db, embeddings)
                },
                |(_, mut db, embeddings)| {
                    for (id, emb) in embeddings {
                        db.set_embedding(id, emb).unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_knn_search,
    benchmark_knn_varying_k,
    benchmark_embedding_dimensions,
    benchmark_set_embedding
);
criterion_main!(benches);
