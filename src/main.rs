use axum::{
    Router,
    response::{Html, Redirect},
    routing::get,
    //routing::get_service,
};
use std::fs;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

const DOC_PATH: &str = "/home/slava/projects/rust/utils/doc_server/target/doc/";
#[tokio::main]
async fn main() {
    let docs_directory =
        "/home/slava/.rustup/toolchains/stable-x86_64-unknown-linux-musl/share/doc/rust/html/";

    let app = Router::new()
        .fallback_service(ServeDir::new(docs_directory))
        .route("/doc/index.html", get(generate_doc_index))
        .nest_service(
            "/doc",
            ServeDir::new(DOC_PATH).append_index_html_on_directories(true),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Rust Documentation Server running at http://localhost:8080");
    println!("Serving files from: {}", docs_directory);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn generate_doc_index() -> Html<String> {
    let mut crates = Vec::new();

    if let Ok(entries) = fs::read_dir(DOC_PATH) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Ignore rustdoc's internal structural asset folders
                    if folder_name != "static.files" && !folder_name.starts_with('.') {
                        crates.push(folder_name.to_string());
                    }
                }
            }
        }
    }

    // Generate a sleek modern dark/light UI dashboard string
    let mut html = String::from(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Local Documentation Hub</title>
            <style>
                body { font-family: system-ui, sans-serif; margin: 40px; background: #1f1f23; color: #e1e1e6; }
                h1 { color: #ff8c00; border-bottom: 1px solid #3a3a40; padding-bottom: 10px; }
                ul { list-style: none; padding: 0; }
                li { background: #2a2a30; margin: 12px 0; padding: 16px; border-radius: 6px; box-shadow: 0 4px 6px rgba(0,0,0,0.2); transition: 0.2s; }
                li:hover { background: #34343d; transform: translateX(5px); }
                a { text-decoration: none; color: #64b5f6; font-weight: bold; display: block; }
            </style>
        </head>
        <body>
            <h1>🦀 Project Documentation Index</h1>
            <p>Dynamically discovered compiled targets inside target/doc:</p>
            <ul>
    "#,
    );

    if crates.is_empty() {
        html.push_str("<li>No crates found. Run <code>cargo doc</code> first!</li>");
    } else {
        for c in crates {
            html.push_str(&format!(
                "<li>📦 <a href='/doc/{}/index.html'>{}</a></li>",
                c, c
            ));
        }
    }

    html.push_str("</ul></body></html>");
    Html(html)
}
