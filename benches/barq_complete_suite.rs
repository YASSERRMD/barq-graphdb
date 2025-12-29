//! Barq Complete Benchmark Suite
//!
//! Comprehensive benchmarks covering all Barq-GraphDB operations at scale.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use barq_graphdb::agent::DecisionRecord;
use barq_graphdb::bench_utils::{generate_random_nodes, generate_random_query, generate_scale_free_edges};
use barq_graphdb::hybrid::HybridParams;
use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use tempfile::TempDir;
use std::time::Duration;

/// Comprehensive write throughput benchmark at scale.
fn barq_write_throughput_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("barq_write_scale");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    for nodes in [1000, 10000, 50000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(nodes), nodes, |b, &n| {
            b.iter_batched(
                || {
                    let dir = TempDir::new().unwrap();
                    let db =
                        BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                    let nodes_data = generate_random_nodes(n, 0);
                    (dir, db, nodes_data)
                },
                |(_, mut db, nodes_data)| {
                    for node in nodes_data {
                        db.append_node(node).unwrap();
                    }
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

/// End-to-end agentic workload benchmark.
fn barq_agentic_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("barq_agentic_workload");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    group.bench_function("full_workflow_1k", |b| {
        b.iter_batched(
            || {
                let dir = TempDir::new().unwrap();
                let db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                (dir, db)
            },
            |(_, mut db)| {
                // 1. Create 1000 nodes with embeddings
                let nodes = generate_random_nodes(1000, 128);
                for node in nodes {
                    let embedding = node.embedding.clone();
                    let id = node.id;
                    db.append_node(node).unwrap();
                    db.set_embedding(id, embedding).unwrap();
                }

                // 2. Create 3000 edges
                let edges = generate_scale_free_edges(1000, 3);
                for (from, to) in edges {
                    let _ = db.add_edge(from, to, "connects");
                }

                // 3. Perform 10 hybrid queries
                for _ in 0..10 {
                    let query = generate_random_query(128);
                    let params = HybridParams::new(0.7, 0.3);
                    let results = db.hybrid_query(&query, 0, 2, 10, params);
                    criterion::black_box(results);
                }

                // 4. Record 10 agent decisions
                for i in 0..10 {
                    let decision = DecisionRecord::new(
                        i as u64,
                        1,
                        0,
                        vec![0, 1, 2],
                        0.9,
                    );
                    db.record_decision(decision).unwrap();
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });
    group.finish();
}

/// Decision recording benchmark.
fn barq_decision_recording(c: &mut Criterion) {
    let mut group = c.benchmark_group("barq_decisions");
    group.sample_size(10);

    for count in [10, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            b.iter_batched(
                || {
                    let dir = TempDir::new().unwrap();
                    let db =
                        BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                    (dir, db)
                },
                |(_, mut db)| {
                    for i in 0..n {
                        let decision = DecisionRecord::new(
                            i as u64,
                            1,
                            0,
                            vec![0, 1, 2, 3, 4],
                            0.85,
                        );
                        db.record_decision(decision).unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Memory efficiency benchmark (measures WAL size).
fn barq_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("barq_memory");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(5);

    for nodes in [10000, 50000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_nodes", nodes)),
            nodes,
            |b, &n| {
                b.iter_batched(
                    || TempDir::new().unwrap(),
                    |dir| {
                        let mut db =
                            BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                        let nodes_data = generate_random_nodes(n, 128);
                        for node in nodes_data {
                            let embedding = node.embedding.clone();
                            let id = node.id;
                            db.append_node(node).unwrap();
                            db.set_embedding(id, embedding).unwrap();
                        }
                        
                        // Report WAL size
                        let wal_path = dir.path().join("wal.log");
                        if let Ok(metadata) = std::fs::metadata(&wal_path) {
                            let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                            println!("WAL size for {} nodes: {:.2}MB", n, size_mb);
                        }
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

/// Concurrent read benchmark (simulated with sequential reads).
fn barq_read_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("barq_read_throughput");
    group.sample_size(20);

    let dir = TempDir::new().unwrap();
    let mut db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();

    let nodes = generate_random_nodes(10000, 128);
    for node in nodes {
        let embedding = node.embedding.clone();
        let id = node.id;
        db.append_node(node).unwrap();
        db.set_embedding(id, embedding).unwrap();
    }

    let edges = generate_scale_free_edges(10000, 3);
    for (from, to) in edges {
        let _ = db.add_edge(from, to, "connects");
    }

    group.bench_function("mixed_reads_10k", |b| {
        b.iter(|| {
            // 5 node lookups
            for _ in 0..5 {
                let id = rand::random::<u64>() % 10000;
                criterion::black_box(db.get_node(id));
            }

            // 3 neighbor queries
            for _ in 0..3 {
                let id = rand::random::<u64>() % 10000;
                criterion::black_box(db.neighbors(id));
            }

            // 2 kNN queries
            for _ in 0..2 {
                let query = generate_random_query(128);
                criterion::black_box(db.knn_search(&query, 10));
            }

            // 1 hybrid query
            let query = generate_random_query(128);
            let params = HybridParams::new(0.7, 0.3);
            criterion::black_box(db.hybrid_query(&query, 0, 2, 10, params));
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    barq_write_throughput_scale,
    barq_agentic_workload,
    barq_decision_recording,
    barq_memory_efficiency,
    barq_read_throughput
);
criterion_main!(benches);
