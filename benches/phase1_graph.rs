//! Phase 1 Graph Benchmarks
//!
//! Benchmarks for graph operations: BFS traversal, neighbor access, edge creation.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use barq_graphdb::bench_utils::{generate_random_nodes, generate_scale_free_edges};
use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use tempfile::TempDir;

/// Benchmark BFS traversal with different graph sizes and hop depths.
fn benchmark_bfs_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("bfs_traversal");
    group.sample_size(10);

    for (nodes, edges, hops) in [(100, 300, 2), (1000, 3000, 3), (10000, 30000, 4)].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}n_{}h", nodes, hops)),
            &(*nodes, *edges, *hops),
            |b, &(n, e, h)| {
                b.iter_batched(
                    || {
                        let dir = TempDir::new().unwrap();
                        let mut db =
                            BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();

                        let nodes_data = generate_random_nodes(n, 0);
                        for node in nodes_data {
                            db.append_node(node).unwrap();
                        }

                        let edges_data = generate_scale_free_edges(n, e / n);
                        for (from, to) in edges_data {
                            let _ = db.add_edge(from, to, "connects");
                        }
                        (dir, db)
                    },
                    |(_, db)| {
                        let result = db.bfs_hops(0, h);
                        criterion::black_box(result);
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

/// Benchmark neighbor access.
fn benchmark_neighbors(c: &mut Criterion) {
    let mut group = c.benchmark_group("neighbor_access");
    group.sample_size(20);

    for nodes in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(nodes), nodes, |b, &n| {
            let dir = TempDir::new().unwrap();
            let mut db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();

            let nodes_data = generate_random_nodes(n, 0);
            for node in nodes_data {
                db.append_node(node).unwrap();
            }

            let edges = generate_scale_free_edges(n, 5);
            for (from, to) in edges {
                let _ = db.add_edge(from, to, "connects");
            }

            b.iter(|| {
                let id = rand::random::<u64>() % n as u64;
                db.neighbors(id)
            });
        });
    }
    group.finish();
}

/// Benchmark edge creation throughput.
fn benchmark_edge_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_creation");
    group.sample_size(10);

    for edge_count in [100, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(edge_count),
            edge_count,
            |b, &e| {
                b.iter_batched(
                    || {
                        let dir = TempDir::new().unwrap();
                        let mut db =
                            BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
                        let nodes = generate_random_nodes(1000, 0);
                        for node in nodes {
                            db.append_node(node).unwrap();
                        }
                        let edges = generate_scale_free_edges(1000, e / 100);
                        (dir, db, edges)
                    },
                    |(_, mut db, edges)| {
                        for (from, to) in edges.into_iter().take(e) {
                            let _ = db.add_edge(from, to, "connects");
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_bfs_traversal,
    benchmark_neighbors,
    benchmark_edge_creation
);
criterion_main!(benches);
