use axum::{Router, routing::get};
use tokio::net::TcpListener;

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/health", get(health));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
