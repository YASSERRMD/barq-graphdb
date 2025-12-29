//! Phase 0 Storage Benchmarks
//!
//! Benchmarks for core storage operations: write throughput, persistence, reload.

use barq_graphdb::bench_utils::generate_random_nodes;
use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::TempDir;

/// Benchmark write throughput for different node counts.
fn benchmark_write_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_throughput");
    group.sample_size(10);

    for nodes in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(nodes), nodes, |b, &n| {
            b.iter_batched(
                || {
                    let dir = TempDir::new().unwrap();
                    let db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                    let nodes_data = generate_random_nodes(n, 0);
                    (dir, db, nodes_data)
                },
                |(_, mut db, nodes_data)| {
                    for node in nodes_data {
                        db.append_node(node).unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark write throughput with embeddings.
fn benchmark_write_with_embeddings(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_with_embeddings");
    group.sample_size(10);

    for nodes in [100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(nodes), nodes, |b, &n| {
            b.iter_batched(
                || {
                    let dir = TempDir::new().unwrap();
                    let db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                    let nodes_data = generate_random_nodes(n, 128);
                    (dir, db, nodes_data)
                },
                |(_, mut db, nodes_data)| {
                    for node in nodes_data {
                        let embedding = node.embedding.clone();
                        let id = node.id;
                        db.append_node(node).unwrap();
                        if !embedding.is_empty() {
                            db.set_embedding(id, embedding).unwrap();
                        }
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark database reload from WAL.
fn benchmark_persistence_reload(c: &mut Criterion) {
    let mut group = c.benchmark_group("persistence_reload");
    group.sample_size(10);

    for nodes in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_nodes", nodes)),
            nodes,
            |b, &n| {
                b.iter_batched(
                    || {
                        let dir = TempDir::new().unwrap();
                        let mut db =
                            BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                        let nodes_data = generate_random_nodes(n, 0);
                        for node in nodes_data {
                            db.append_node(node).unwrap();
                        }
                        drop(db);
                        dir
                    },
                    |dir| {
                        let db =
                            BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                        assert_eq!(db.node_count(), n);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

/// Benchmark single node lookup.
fn benchmark_node_lookup(c: &mut Criterion) {
    c.bench_function("node_lookup_10k", |b| {
        let dir = TempDir::new().unwrap();
        let mut db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
        let nodes = generate_random_nodes(10000, 0);
        for node in nodes {
            db.append_node(node).unwrap();
        }

        b.iter(|| {
            let id = rand::random::<u64>() % 10000;
            db.get_node(id)
        });
    });
}

criterion_group!(
    benches,
    benchmark_write_throughput,
    benchmark_write_with_embeddings,
    benchmark_persistence_reload,
    benchmark_node_lookup
);
criterion_main!(benches);
