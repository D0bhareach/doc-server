use axum::{Router, routing::get_service};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let docs_directory =
        "/home/slava/.rustup/toolchains/stable-x86_64-unknown-linux-musl/share/doc/rust/html/";
    let app = Router::new().fallback_service(get_service(ServeDir::new(docs_directory)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Rust Documentation Server running at http://localhost:8080");
    println!("Serving files from: {}", docs_directory);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
