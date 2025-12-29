//! Phase 3 Hybrid Query Benchmarks
//!
//! Benchmarks for hybrid queries combining vector similarity and graph traversal.

use barq_graphdb::bench_utils::{
    generate_random_nodes, generate_random_query, generate_scale_free_edges,
};
use barq_graphdb::hybrid::HybridParams;
use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::TempDir;

/// Benchmark hybrid query with different graph sizes.
fn benchmark_hybrid_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_query");
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

            let edges = generate_scale_free_edges(n, 3);
            for (from, to) in edges {
                let _ = db.add_edge(from, to, "connects");
            }

            let params = HybridParams::new(0.7, 0.3);

            b.iter(|| {
                let query = generate_random_query(128);
                let results = db.hybrid_query(&query, 0, 2, 10, params.clone());
                criterion::black_box(results);
            });
        });
    }
    group.finish();
}

/// Benchmark hybrid query with varying hop depths.
fn benchmark_hybrid_varying_hops(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_varying_hops");
    group.sample_size(10);

    // Setup 3000 node database
    let dir = TempDir::new().unwrap();
    let mut db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();

    let nodes_data = generate_random_nodes(3000, 128);
    for node in nodes_data {
        let embedding = node.embedding.clone();
        let id = node.id;
        db.append_node(node).unwrap();
        db.set_embedding(id, embedding).unwrap();
    }

    let edges = generate_scale_free_edges(3000, 3);
    for (from, to) in edges {
        let _ = db.add_edge(from, to, "connects");
    }

    for hops in [1, 2, 3, 4].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(hops), hops, |b, &h| {
            let params = HybridParams::new(0.7, 0.3);
            b.iter(|| {
                let query = generate_random_query(128);
                let results = db.hybrid_query(&query, 0, h, 10, params.clone());
                criterion::black_box(results);
            });
        });
    }
    group.finish();
}

/// Benchmark hybrid query with varying alpha/beta weights.
fn benchmark_hybrid_params(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_params");
    group.sample_size(10);

    // Setup 2000 node database
    let dir = TempDir::new().unwrap();
    let mut db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();

    let nodes_data = generate_random_nodes(2000, 128);
    for node in nodes_data {
        let embedding = node.embedding.clone();
        let id = node.id;
        db.append_node(node).unwrap();
        db.set_embedding(id, embedding).unwrap();
    }

    let edges = generate_scale_free_edges(2000, 3);
    for (from, to) in edges {
        let _ = db.add_edge(from, to, "connects");
    }

    for (alpha, beta) in [(1.0, 0.0), (0.7, 0.3), (0.5, 0.5), (0.3, 0.7), (0.0, 1.0)].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("a{}_b{}", alpha, beta)),
            &(*alpha, *beta),
            |b, &(a, bt)| {
                let params = HybridParams::new(a, bt);
                b.iter(|| {
                    let query = generate_random_query(128);
                    let results = db.hybrid_query(&query, 0, 2, 10, params.clone());
                    criterion::black_box(results);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark hybrid query with varying k values.
fn benchmark_hybrid_varying_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_varying_k");
    group.sample_size(10);

    let dir = TempDir::new().unwrap();
    let mut db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();

    let nodes_data = generate_random_nodes(2000, 128);
    for node in nodes_data {
        let embedding = node.embedding.clone();
        let id = node.id;
        db.append_node(node).unwrap();
        db.set_embedding(id, embedding).unwrap();
    }

    let edges = generate_scale_free_edges(2000, 3);
    for (from, to) in edges {
        let _ = db.add_edge(from, to, "connects");
    }

    for k in [5, 10, 25, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(k), k, |b, &k_val| {
            let params = HybridParams::new(0.7, 0.3);
            b.iter(|| {
                let query = generate_random_query(128);
                let results = db.hybrid_query(&query, 0, 2, k_val, params.clone());
                criterion::black_box(results);
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_hybrid_query,
    benchmark_hybrid_varying_hops,
    benchmark_hybrid_params,
    benchmark_hybrid_varying_k
);
criterion_main!(benches);
