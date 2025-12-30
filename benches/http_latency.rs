use axum::{
    routing::{get, post},
    Router,
};
use barq_graphdb::{
    api,
    storage::{BarqGraphDb, DbOptions},
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

async fn start_test_server() -> (String, TempDir) {
    let dir = TempDir::new().unwrap();
    let db = BarqGraphDb::open(DbOptions::new(dir.path().to_path_buf())).unwrap();
    let state = Arc::new(Mutex::new(db));

    let app = Router::new()
        .route("/health", get(api::health_check))
        .route("/nodes", get(api::list_nodes))
        .route("/nodes/:id", get(api::get_node))
        .route("/nodes", post(api::create_node))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give it a split second to ensure listening (though bind is synchronous for port allocation)
    tokio::time::sleep(Duration::from_millis(50)).await;

    (url, dir)
}

fn benchmark_http_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (base_url, _dir) = rt.block_on(start_test_server());
    let client = reqwest::Client::new();

    let mut group = c.benchmark_group("http_latency");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10); // Minimum sample size

    // 1. Benchmark POST /nodes
    group.bench_function("post_node", |b| {
        b.to_async(&rt).iter_custom(|iters| {
            let client = client.clone(); // efficient clone
            let url = format!("{}/nodes", base_url);
            async move {
                let start = std::time::Instant::now();
                for i in 0..iters {
                    let body = serde_json::json!({
                        "id": i,
                        "label": "Benchmark Node",
                        "embedding": vec![0.0; 128] // Include embedding overhead
                    });
                    let _res = client.post(&url).json(&body).send().await.unwrap();
                }
                start.elapsed()
            }
        });
    });

    // 2. Prepare data for GET
    // We already inserted 'iters' nodes in the previous step? No, `iter_custom` runs measurement.
    // Let's explicitly insert a node 99999 for GET.
    rt.block_on(async {
        let body = serde_json::json!({
            "id": 99999,
            "label": "Get Target",
            "embedding": vec![1.0; 128]
        });
        client
            .post(&format!("{}/nodes", base_url))
            .json(&body)
            .send()
            .await
            .expect("Failed to seed node 99999");
    });

    // 3. Benchmark GET /nodes/{id}
    group.bench_function("get_node", |b| {
        b.to_async(&rt).iter(|| async {
            let url = format!("{}/nodes/99999", base_url);
            let res = client.get(&url).send().await.unwrap();
            assert!(res.status().is_success());
        });
    });

    // 4. Benchmark Health Check (Baseline overhead)
    group.bench_function("get_health", |b| {
        b.to_async(&rt).iter(|| async {
            let url = format!("{}/health", base_url);
            let res = client.get(&url).send().await.unwrap();
            assert!(res.status().is_success());
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_http_latency);
criterion_main!(benches);
