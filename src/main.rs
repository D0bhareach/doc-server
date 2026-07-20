mod files;
use axum::Router;
use clap::Parser;
use std::env;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to root dir where "target/doc" will be automatically discovered.
    /// By default doc-server try to get $HOME dir, if fail to find $HOME doc-server will
    /// try to get current dir if nothing succeed doc-server will exit with error.
    #[arg(short = 'w', long)]
    pub path: Option<PathBuf>,

    /// Port for local http server.
    #[arg(short = 'p', long, default_value_t = 8080)]
    pub port: u16,

    /// IP address for server.
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    pub host: IpAddr,

    /// Verbose logging to standard output. Will print DEBUG and INFO messages
    #[arg(short = 'v', default_value_t = false)]
    pub verbose: bool,
}

/// create index.html for out server
/// toolchain_path - Option value can be not discovered, path to rust docs
/// toolchain_url - const value for route for rust docs
/// doc_url - const value for root route for all discoverd docs
/// projects - vector of DocProject struct for all discovered project
fn create_index_page_content(
    toolchain_path: Option<&Path>,
    toolchain_url: &str,
    doc_url: &str,
    projects: &[files::DocProject],
) -> String {
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

    for proto in projects.iter() {
        // Формируем ссылку на index.html каждого найденного крейта
        if proto.crates.is_empty() {
            html.push_str(&format!(
                "<li><a href='{}/{}/{}/index.html'>{}</a></li>",
                doc_url, proto.name, proto.name, proto.name
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
                    "<li><a href='{}/{}/{}/index.html'>📦 {}</a></li>",
                    doc_url, proto.name, krate, krate
                ));
            }
            html.push_str("</ul></details>");
        }
    }

    html.push_str("</ul></body></html>");
    html
}

/// graceful shutdown listening for signals
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("[ERROR] in listener for signal Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("[ERROR] can not register listener for signal SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { println!("\ngot Ctrl+C signal, graceful shutdown..."); },
        _ = terminate => { println!("\ngot SIGTERM signal, graceful shutdown..."); },
    }
}

/// url for app router for toolchain
static TOOLCHAIN_URL: &str = "/toolchain_doc/index.html";

/// nested service route for all static files in the folder
static TOOLCHAIN_ROOT_URL: &str = "/toolchain_doc";

/// root route for all documentation that are found
static DOC_URL: &str = "/doc";

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let home_dir: PathBuf = args
        .path
        .or_else(|| directories::UserDirs::new().map(|dirs| dirs.home_dir().to_path_buf()))
        .or_else(|| Some(env::current_dir().ok()?))
        .unwrap_or_else(|| panic!("[ERROR] can not get path for $HOME directory"));

    let toolchain_doc_path = files::toolchain::find_rust_docs_path()
        .inspect_err(|e| {
            if args.verbose {
                eprintln!("[DEBUG] can not find toolchain path. {}", e);
            }
        })
        .ok();

    // let toolchain_doc_path =
    // " /home/slava/.rustup/toolchains/stable-x86_64-unknown-linux-musl/share/doc/rust/html";
    //
    let found_docs = files::find_docs(&home_dir);

    let index_file_content = create_index_page_content(
        toolchain_doc_path.as_deref(),
        TOOLCHAIN_URL,
        DOC_URL,
        &found_docs,
    );

    // index.html file must exist if not panic
    let index_file_path =
        files::prepare_cache_index_file(&index_file_content).unwrap_or_else(|err| {
            panic!("[ERROR] creating index.html file in cache dir. {}", err);
        });

    let shared_projects = Arc::new(found_docs);

    let mut app = Router::new();

    if let Some(toolchain_path) = &toolchain_doc_path {
        app = app.nest_service(
            TOOLCHAIN_ROOT_URL,
            ServeDir::new(toolchain_path).append_index_html_on_directories(true),
        );
    };

    // dynamically create my index page
    app = app.fallback_service(ServeFile::new(&index_file_path));

    for project in shared_projects.iter() {
        let absolute_path = match project.doc_path.canonicalize() {
            Ok(path) => path,
            Err(e) => {
                if args.verbose {
                    eprintln!("[DEBUG] can not find path {:?}: {}", project.doc_path, e);
                }
                continue;
            }
        };
        // create service to server all files in docs
        let root_route = format!("{}/{}", DOC_URL, &project.name);
        let route_service = ServeDir::new(&absolute_path).append_index_html_on_directories(true);
        app = app.nest_service(&root_route, route_service);
    }
    let addr = std::net::SocketAddr::new(args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("error binding TCP Listener");

    println!("🚀 Doc Server is  running at http://localhost:8080");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("error serving app");

    index_file_path.parent().and_then(|dir| {
        if dir.exists() {
            let _ = files::cleanup_temp_dir(dir).inspect_err(|err| {
                eprintln!("[Error] can not remove cached directory: {}", err);
            });
            Some(())
        } else {
            panic!("[Error] can not get path to cached directory");
        }
    });
}
