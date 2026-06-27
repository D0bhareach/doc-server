use axum::{Router, response::Redirect, routing::get, routing::get_service};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let docs_directory = // "home/slava/projects/rust/utils/doc_server/target/doc/";
    "/home/slava/.rustup/toolchains/stable-x86_64-unknown-linux-musl/share/doc/rust/html/";
    // need to create services for dirs and create and points for them
    // let app = Router::new().fallback_service(get_service(ServeDir::new(docs_directory)));
    let app = Router::new()
        .fallback_service(ServeDir::new(docs_directory))
        .route(
            "/axum",
            get(|| async { Redirect::permanent("/doc/axum/index.html") }),
        )
        .route(
            "/axum-core",
            get(|| async { Redirect::permanent("/doc/axum_core/index.html") }),
        )
        .nest_service(
            "/doc",
            ServeDir::new("/home/slava/projects/rust/utils/doc_server/target/doc/")
                .append_index_html_on_directories(true),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Rust Documentation Server running at http://localhost:8080");
    println!("Serving files from: {}", docs_directory);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
