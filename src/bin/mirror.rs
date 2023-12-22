use axum::{
    routing::get,
    Router,
};
use std::time::Duration;

fn main() {
    let app = Router::new()
        .route("/api/latest", get(noop));

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        })
}

// basic handler that responds with a static string
async fn noop() -> &'static str {
    tokio::time::sleep(Duration::new(0, 100_000_000)).await;
    "Hello, world, goodbyte!"
}
