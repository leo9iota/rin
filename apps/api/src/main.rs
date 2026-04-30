use axum::{Router, routing::get};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Rin API server...");

    let app = Router::new().route("/", get(|| async { "Rin API running" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
