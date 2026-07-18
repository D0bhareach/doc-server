use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum FilesError {
    #[error("Filesystem I/O failure: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Requested path not found: {0}")]
    NotFound(PathBuf),
}
