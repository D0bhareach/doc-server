use axum::{
    Router,
    extract::Path,
    response::Html,
    routing::{get, get_service},
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;

// CHANGE THIS: Point it to the folder containing all your Rust projects
const WORKSPACE_DIR: &str = "/home/user/projects/rust";

#[tokio::main]
async fn main() {
    let app = Router::new()
        // 1. Serve the dynamic HTML index page at the root
        .route("/", get(render_index))
        // 2. Dynamic route to serve specific project docs without copying
        .nest_service("/docs/:project_name", get_service(ServeDir::new("")))
        // We override the Nested Service path dynamically via a middleware hack or fallback
        .fallback(handle_docs);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Dynamic Multi-Doc Server running at http://localhost:8080");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler to generate the central hub HTML page dynamically
async fn render_index() -> Html<String> {
    let mut projects = Vec::new();
    let workspace = PathBuf::from(WORKSPACE_DIR);

    // Scan the workspace directory
    if let Ok(mut entries) = std::fs::read_dir(workspace) {
        while let Some(Ok(entry)) = entries.next() {
            let path = entry.path();
            if path.is_dir() {
                let project_name = path.file_name().unwrap().to_string_lossy().into_owned();

                // Check if this project has a built documentation folder
                let doc_path = path.join("target").join("doc");
                if doc_path.join("index.html").exists() || doc_path.exists() {
                    projects.push(project_name);
                }
            }
        }
    }

    // Generate HTML string
    let mut html = String::from(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Rust Local Documentation Hub</title>
            <style>
                body { font-family: sans-serif; margin: 40px; background: #f4f4f9; color: #333; }
                h1 { color: #e05d44; }
                ul { list-style-type: none; padding: 0; }
                li { background: white; margin: 10px 0; padding: 15px; border-radius: 5px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
                a { text-decoration: none; color: #0066cc; font-weight: bold; font-size: 1.1em; }
                a:hover { color: #004499; }
            </style>
        </head>
        <body>
            <h1>🦀 Active Rust Projects Documentation</h1>
            <p>Automatically discovering built docs in your workspace...</p>
            <ul>
    "#,
    );

    if projects.is_empty() {
        html.push_str("<li>No projects with compiled documentation found. Run <code>cargo doc</code> inside a project directory!</li>");
    } else {
        for project in projects {
            // Point link to our dynamic proxy endpoint
            html.push_str(&format!(
                "<li>📁 <a href='/docs/{}/index.html'>{}</a></li>",
                project, project
            ));
        }
    }

    html.push_str("</ul></body></html>");
    Html(html)
}

// Fallback router that intercepts /docs/project_name/... requests
// and maps them directly to the real target/doc directory on your VM disk.
async fn handle_docs(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    let path_str = uri.path();

    // Parse the project name out of the URL string (/docs/my_project/...)
    let parts: Vec<&str> = path_str.split('/').collect();
    if parts.len() >= 3 && parts[1] == "docs" {
        let project_name = parts[2];

        // Reconstruct the real filesystem path inside your workspace
        let mut real_path = PathBuf::from(WORKSPACE_DIR);
        real_path.push(project_name);
        real_path.push("target");
        real_path.push("doc");

        // Append the rest of the requested file path (e.g., index.html, main.js)
        let remaining_path = parts[3..].join("/");
        let file_to_serve = real_path.join(remaining_path);

        if file_to_serve.exists() {
            return get_service(ServeDir::new(real_path))
                .call(
                    axum::http::Request::builder()
                        .uri(uri)
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .into_response();
        }
    }

    axum::http::StatusCode::NOT_FOUND.into_response()
}
