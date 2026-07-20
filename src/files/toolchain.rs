use crate::files::FilesError;
use directories::BaseDirs;
use std::path::PathBuf;
use std::process::Command;

pub fn find_rust_docs_path() -> Result<PathBuf, FilesError> {
    // prepare PathBuf with capacity to avoid reallocation in the heap.
    // standard rustup doc path without host name is about 90 - 120 char long
    // 150 bytes shall be way enough
    let mut path = PathBuf::with_capacity(150);

    // ignore errors and chain attempts to get path
    // try to exexute rustup CLI
    if let Ok(output) = Command::new("rustup").args(["doc", "--path"]).output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            path.push(path_str);
            // rustup doc --path return path to index.html
            if path.pop() {
                return Ok(path);
            }
        }
    }
    // if above not working - fallback to env::var
    if let Ok(rustup_home) = std::env::var("RUSTUP_HOME") {
        path.push(rustup_home);
        path.push("toolchains");
        if path.exists() {
            // Если папка существует, пытаемся найти там документацию
            if let Ok(path) = find_docs_in_toolchains(&path) {
                return Ok(path);
            }
        }
    }

    // fallback -> check ~/.rustup in $HOME
    if let Some(base_dirs) = BaseDirs::new() {
        path.push(base_dirs.home_dir());
        path.push(".rustup");
        path.push("toolchains");
        if path.exists() {
            if let Ok(path) = find_docs_in_toolchains(&path) {
                return Ok(path);
            }
        }
    }

    Err(FilesError::NotFound(
        "all chained searches for toolchain doc path fail".to_string(),
    ))
}

fn find_docs_in_toolchains(toolchains_dir: &PathBuf) -> Result<PathBuf, FilesError> {
    if let Ok(entries) = std::fs::read_dir(toolchains_dir) {
        for entry in entries.flatten() {
            let doc_path = entry.path().join("share/doc/rust/html");
            if doc_path.exists() {
                return Ok(doc_path);
            }
        }
    }
    Err(FilesError::NotFound(
        "search for rust docs in $RUST_HOME dir fail".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_rust_docs() {
        let docs = find_rust_docs_path();
        assert!(docs.is_ok(), "Fail to find Rust Docs!");
        println!("Rust Docs found at: {:?}", docs.unwrap());
    }
}
