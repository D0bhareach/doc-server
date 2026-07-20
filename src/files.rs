use directories::ProjectDirs;
use ignore::WalkBuilder;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub mod errors;
pub mod toolchain;
use self::errors::FilesError;

pub fn prepare_cache_index_file(html_content: &str) -> Result<PathBuf, FilesError> {
    // /doc-server = 11, /index.html = 11, /home = 5, /.cache = 7 so total = 34
    // give total = 70 double the size 35 bytes not a huge over take.
    const TOTAL_CAPACITY: usize = 70;
    let mut path = PathBuf::with_capacity(TOTAL_CAPACITY);

    let Some(project_dirs) = ProjectDirs::from("", "", "doc_server") else {
        return Err(errors::FilesError::Other(
            "unable to create ProjectsDirs".to_string(),
        ));
    };

    let cache = project_dirs.cache_dir();
    path.push(cache);

    fs::create_dir_all(&path)?;

    // reuse path
    path.push("index.html");

    let mut file = fs::File::create(&path)?;
    file.write_all(html_content.as_bytes())?;

    Ok(path)
}

pub fn cleanup_temp_dir(tmp_file_path: &Path) -> Result<(), FilesError> {
    fs::remove_dir_all(tmp_file_path).map_err(FilesError::IoError)
}

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
