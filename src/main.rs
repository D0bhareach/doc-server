use axum::{
    Router,
    response::Html,
    routing::{get, get_service},
};
use std::fs;
use tower_http::services::ServeDir;

// TODO:  I need to build this paths dinamically
const DOC_PATH: &str = "/home/slava/projects/rust/utils/doc_server/target/doc/";
const TOOLCHAIN_DOC_PATH: &str =
    "/home/slava/.rustup/toolchains/stable-x86_64-unknown-linux-musl/share/doc/rust/html/";

#[tokio::main]
async fn main() {
    let doc_service = ServeDir::new(DOC_PATH).append_index_html_on_directories(true);
    let toolchan_doc_service =
        ServeDir::new(TOOLCHAIN_DOC_PATH).append_index_html_on_directories(true);

    let app = Router::new()
        .nest_service("/doc", get_service(doc_service))
        .nest_service("/toolchain_doc", get_service(toolchan_doc_service))
        .route("/", get(generate_doc_index));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("🚀 Doc Server is  running at http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}

// Generates your dashboard (unchanged)
async fn generate_doc_index() -> Html<String> {
    let mut crates = Vec::new();

    if let Ok(entries) = fs::read_dir(DOC_PATH) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                    if folder_name != "static.files" && !folder_name.starts_with('.') {
                        crates.push(folder_name.to_string());
                    }
                }
            }
        }
        crates.sort();
    }

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
                a,summary { text-decoration: none; color: #64b5f6; font-weight: bold; display: block; }
                summary {list-style: none;}
                summary::-webkit-details-marker { display: none; }
                summary {font-size: 1.2rem; cursor: pointer; user-select: none;}
                summary::before { content: "▶"; font-size: 1.1rem; line-height: 1.1rem; color: #64b5f6;  display: inline-block; transform-origin: 10% 50%;  transition: transform 0.2s ease-in-out; padding-right: 1rem; }
                details[open] > summary::before { transform: rotate(90deg); }
            </style>
        </head>
        <body>
            <h1>🦀 Documentation Index</h1>
            <p>Dynamically discovered compiled targets inside target/doc:</p>
            <ul>
                <li><a href='/toolchain_doc/index.html'>📦 Rust Toolchain Docs</a></li>
            </ul>
            <details>
                <summary>doc_server</summary>
                <ul>
                    <li><a href='/doc/doc_server/index.html'>📦 doc_server</a>
    "#,
    );

    if crates.is_empty() {
        html.push_str("<li>No crates found. Run <code>cargo doc</code> first!</li>");
    } else {
        for krate in crates {
            html.push_str(&format!(
                "<li><a href='/doc/{}/index.html'>📦 {}</a></li>",
                krate, krate
            ));
        }
    }

    html.push_str("</ul></details></body></html>");
    Html(html)
}
