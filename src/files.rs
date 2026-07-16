use ignore::WalkBuilder;
use std::env;
use std::fs;
use std::io::{self, Error as IO_Error, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub fn prepare_cache_index_file(html_content: &str) -> io::Result<PathBuf> {
    let mut base_path = if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg_cache)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache")
    } else {
        return Err(IO_Error::new(
            io::ErrorKind::Other,
            "[Error] problem accessing user's cache dir",
        ));
    };

    // 2. Добавляем имя нашего приложения
    base_path.push("doc_server");

    // 3. Создаем директорию (аналог mkdir -p)
    fs::create_dir_all(&base_path)?;

    // 4. Формируем путь к файлу: ~/.cache/doc_server/index.html
    let file_path = base_path.join("index.html");

    // 5. Записываем сгенерированный HTML
    let mut file = fs::File::create(&file_path)?;
    file.write_all(html_content.as_bytes())?;

    Ok(file_path)
}

pub fn cleanup_temp_dir(tmp_file_path: &Path) {
    let res = tmp_file_path.parent().and_then(|parent| {
        if parent.exists() {
            fs::remove_dir_all(parent).ok()
        } else {
            None
        }
    });

    if res.is_none() {
        eprintln!("doc-server can not remove cached index.html file directory.");
    }
}

fn main() {
    let html_content = "<!DOCTYPE html><html><body><h1>Rust Doc Server Index</h1></body></html>";

    match prepare_xdg_resources(html_content) {
        Ok(path) => {
            println!(
                "[УСПЕХ] Индексный файл создан по стандарту XDG: {}",
                path.display()
            );
            // Этот path мы передаем в Axum: ServeFile::new(path)
        }
        Err(e) => {
            eprintln!("[ОШИБКА] Не удалось создать файл в директории кэша: {}", e);
        }
    }
}

pub fn get_toolchain_doc_path() -> Option<PathBuf> {
    let home = env::var("RUSTUP_HOME").ok()?;
    let toolchain = env::var("RUSTUP_TOOLCHAIN").ok()?;

    let mut path = PathBuf::from(home);
    path.push("toolchains");
    path.push(toolchain);
    path.push("share/doc/rust/html");

    Some(path)
}

// need to sort lists of this struct by name.
#[derive(Debug, Clone)]
pub struct DocProject {
    pub name: String,
    pub doc_path: PathBuf,
    pub crates: Vec<String>,
}

pub fn find_docs(root_dir: &Path) -> Vec<DocProject> {
    let (tx, rx) = mpsc::channel();

    let walker = WalkBuilder::new(root_dir)
        .hidden(true)
        .git_ignore(false)
        .build_parallel();

    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                let path = entry.path();

                if path.is_dir() && path.ends_with("target/doc") {
                    // need to build sub_dirs vector here

                    let mut crates = Vec::new();

                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                let index_file = path.join("index.html");
                                if index_file.is_file() {
                                    if let Some(folder_name) =
                                        path.file_name().and_then(|n| n.to_str())
                                    {
                                        crates.push(folder_name.to_string());
                                    }
                                }
                            }
                        }
                        crates.sort();
                    }
                    if let Some(project_root) = path.parent().and_then(|p| p.parent()) {
                        if let Some(project_name) =
                            project_root.file_name().and_then(|n| n.to_str())
                        {
                            let project = DocProject {
                                name: project_name.to_string(),
                                doc_path: path.to_path_buf(),
                                crates,
                            };
                            let _ = tx.send(project);
                        }
                    }
                }
            };
            ignore::WalkState::Continue
        })
    });

    // end of thread walker
    drop(tx);
    let mut projects: Vec<DocProject> = rx.into_iter().collect();
    projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    projects
}
