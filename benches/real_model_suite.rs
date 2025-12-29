//! Benchmarks using a real embedding model (all-MiniLM-L6-v2) via fastembed.
//!
//! This suite demonstrates performance on real-world semantic data rather than random vectors.
//! Note: The model files are downloaded to 'fastembed_cache/' which is .gitignored.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use barq_graphdb::storage::{BarqGraphDb, DbOptions};
use barq_graphdb::Node;
use fastembed::{InitOptions, TextEmbedding, EmbeddingModel};
use tempfile::TempDir;
use std::time::Duration;

// A small corpus of real-world sentences for semantic search
const REAL_DATASET: &[&str] = &[
    "The quick brown fox jumps over the lazy dog.",
    "Artificial intelligence is transforming the technology industry.",
    "Graph databases are excellent for managing connected data.",
    "Vector databases enable semantic search capabilities.",
    "Rust provides memory safety without garbage collection.",
    "The stock market saw a significant rise today due to tech earnings.",
    "Climate change is a pressing global issue requiring immediate action.",
    "SpaceX successfully launched a new batch of Starlink satellites.",
    "Deep learning models require vast amounts of data for training.",
    "Hybrid search combines keyword matching with semantic understanding.",
    "Distributed systems face challenges in consistency and availability.",
    "The James Webb Space Telescope has captured stunning images of the universe.",
    "Quantum computing promises to solve problems intractable for classical computers.",
    "Cybersecurity threats are evolving with more sophisticated phishing attacks.",
    "Renewable energy sources like solar and wind are becoming cheaper.",
    "The human genome project has improved our understanding of genetics.",
    "Autonomous vehicles are being tested in major cities worldwide.",
    "Blockchain technology underpins cryptocurrencies like Bitcoin and Ethereum.",
    "Microservices architecture improves scalability but increases complexity.",
    "Cloud computing allows businesses to scale infrastructure on demand.",
];

const QUERY_TEXTS: &[&str] = &[
    "machine learning trends",
    "sustainable energy power",
    "programming languages performance",
    "space exploration news",
    "database technology",
];

/// Initialize the embedding model.
/// Initialize the embedding model.
fn init_model() -> TextEmbedding {
    let mut options = InitOptions::default();
    options.model_name = EmbeddingModel::AllMiniLML6V2;
    options.show_download_progress = true;
    options.cache_dir = std::path::PathBuf::from("fastembed_cache");
    
    TextEmbedding::try_new(options).expect("Failed to initialize embedding model")
}

/// Benchmark Vector Search with Real Embeddings
fn benchmark_real_vector_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_vector_search");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    // 1. Initialize Model
    println!("Initializing embedding model (this may download files)...");
    let mut model = init_model();

    // 2. Generate Embeddings for Dataset
    // Replicate dataset to reach a reasonable size (e.g., ~2000 nodes)
    let replication_factor = 100;
    let docs: Vec<String> = (0..replication_factor)
        .flat_map(|i| REAL_DATASET.iter().map(move |s| format!("{} [{}]", s, i)))
        .collect();
    
    println!("Generating embeddings for {} documents...", docs.len());
    let embeddings = model.embed(docs.clone(), None).expect("Embedding generation failed");
    
    // 3. Generate Embeddings for Queries
    let query_embeddings = model.embed(QUERY_TEXTS.to_vec(), None).expect("Query embedding failed");

    // 4. Setup Database
    for size in [100, 1000, 2000].iter() {
        if *size > docs.len() { continue; }
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &n| {
            b.iter_batched(
                || {
                    let dir = TempDir::new().unwrap();
                    let mut db = BarqGraphDb::open(
                        DbOptions::new(dir.path().to_path_buf())
                    ).unwrap();

                    // Insert N nodes with embeddings
                    for i in 0..n {
                        let node = Node {
                            id: i as u64,
                            label: docs[i].clone(),
                            embedding: embeddings[i].clone(),
                            edges: vec![],
                            timestamp: 0,
                            agent_id: None,
                            rule_tags: vec![],
                        };
                        db.append_node(node).unwrap();
                        db.set_embedding(i as u64, embeddings[i].clone()).unwrap();
                    }
                    (dir, db)
                },
                |(_, db)| {
                    // Pick a random query vector
                    let q_idx = rand::random::<usize>() % query_embeddings.len();
                    let q_vec = &query_embeddings[q_idx];
                    
                    let results = db.knn_search(q_vec, 10);
                    assert!(!results.is_empty());
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark Graph Traversal on a "Semantic Graph"
/// Edges are created between semantically similar nodes (simulated by dataset adjacency in this case).
fn benchmark_real_graph_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_graph_traversal");
    group.sample_size(10);
    
    // Use the same dataset logic
    let mut model = init_model();
    let replication_factor = 50; // 1000 nodes
    let docs: Vec<String> = (0..replication_factor)
        .flat_map(|i| REAL_DATASET.iter().map(move |s| format!("{} [{}]", s, i)))
        .collect();
        
    let embeddings = model.embed(docs.clone(), None).expect("Embedding generation failed");

    group.bench_function("semantic_graph_bfs_1k", |b| {
        b.iter_batched(
            || {
                let dir = TempDir::new().unwrap();
                let mut db = BarqGraphDb::open(
                    DbOptions::new(dir.path().to_path_buf())
                ).unwrap();

                // 1. Insert Nodes
                for i in 0..docs.len() {
                    let node = Node {
                        id: i as u64,
                        label: docs[i].clone(),
                        embedding: embeddings[i].clone(),
                        edges: vec![],
                        timestamp: 0,
                        agent_id: None,
                        rule_tags: vec![],
                    };
                    db.append_node(node).unwrap();
                }

                // 2. Create "Semantic" Edges
                // For this benchmark, we'll link texts that originate from the same template sentence
                // (simulating a cluster of related documents)
                let stride = REAL_DATASET.len();
                for i in 0..docs.len() {
                    // Connect to next instance of similar topic (if exists)
                    if i + stride < docs.len() {
                         db.add_edge(i as u64, (i + stride) as u64, "semantically_related").unwrap();
                    }
                    // Connect to previous for bidirectional flow
                    if i >= stride {
                        db.add_edge(i as u64, (i - stride) as u64, "semantically_related").unwrap();
                    }
                    // Add some random cross-links for complexity
                    if i % 7 == 0 {
                         let target = (i + 3) % docs.len();
                         db.add_edge(i as u64, target as u64, "cross_reference").unwrap();
                    }
                }
                (dir, db)
            },
            |(_, db)| {
                // BFS Traversal
                let start_node = 0;
                let hops = 3;
                let visited = db.bfs_hops(start_node, hops);
                assert!(!visited.is_empty());
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, benchmark_real_vector_search, benchmark_real_graph_traversal);
criterion_main!(benches);
