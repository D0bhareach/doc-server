mod files;
use axum::{Router, response::Html, routing::get};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

async fn index_page(
    toolchain_path: Option<PathBuf>,
    toolchain_url: &str,
    projects: Arc<Vec<files::DocProject>>,
) -> Html<String> {
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
            <ul>"#,
    );

    if let Some(_) = toolchain_path {
        html.push_str(&format!(
            "<li><a href='{}'>📦 Rust Toolchain Docs</a></li>",
            toolchain_url
        ));
    };

    // TODO: project are not sorted.
    for proto in projects.iter() {
        // Формируем ссылку на index.html каждого найденного крейта
        if proto.crates.is_empty() {
            html.push_str(&format!(
                "<li><a href='/doc/{}/{}/index.html'>{}</a></li>",
                proto.name, proto.name, proto.name
            ));
        } else {
            html.push_str(&format!(
                "<li><details>
                <summary>{}</summary>
                <ul>",
                proto.name
            ));

            for krate in &proto.crates {
                html.push_str(&format!(
                    "<li><a href='/doc/{}/{}/index.html'>📦 {}</a></li>",
                    proto.name, krate, krate
                ));
            }
            html.push_str("</ul></details>");
        }
    }

    html.push_str("</ul></body></html>");
    Html(html)
}

const TOOLCHAIN_URL: &str = "/toolchain_doc/index.html";
static TOOLCHAIN_ROOT_URL: &str = "/toolchain_doc";
#[tokio::main]
async fn main() {
    let home_dir = env::home_dir().expect("error get $HOME path");
    let toolchain_doc_path = files::get_toolchain_doc_path();
    let found_docs = files::find_docs(&home_dir);

    let shared_projects = Arc::new(found_docs);
    let projects_for_route = Arc::clone(&shared_projects);

    let mut app = Router::new();
    if let Some(toolchain_path) = &toolchain_doc_path {
        app = app.nest_service(
            TOOLCHAIN_ROOT_URL,
            ServeDir::new(toolchain_path).append_index_html_on_directories(true),
        );
    };

    // dynamically create my index page
    app = app.route(
        "/",
        get(move || index_page(toolchain_doc_path, TOOLCHAIN_URL, projects_for_route)),
    );

    // Шаг 3: Динамически регистрируем каждую найденную папку doc в веб-сервере
    for project in shared_projects.iter() {
        let absolute_path = match project.doc_path.canonicalize() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("[ERROR] can not find path {:?}: {}", project.doc_path, e);
                continue;
            }
        };
        // create service to server all files in docs
        let root_route = format!("/doc/{}", &project.name);
        let route_service = ServeDir::new(&absolute_path).append_index_html_on_directories(true);
        app = app.nest_service(&root_route, route_service);
    }
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("error binding TCP Listener");

    println!("🚀 Doc Server is  running at http://localhost:8080");
    axum::serve(listener, app).await.expect("error serving app");
}
