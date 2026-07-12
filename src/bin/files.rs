use std::env;
use std::fs::read_dir;
use std::io::{Error, ErrorKind, Result as IO_Result};
use std::path::{Path, PathBuf};

fn get_toolchain_doc_path() -> Option<PathBuf> {
    let home = env::var("RUSTUP_HOME").ok()?;
    let toolchain = env::var("RUSTUP_TOOLCHAIN").ok()?;

    let mut path = PathBuf::from(home);
    path.push("toolchains");
    path.push(toolchain);
    path.push("share/doc/rust/html");

    Some(path)
}

fn walk_dir_recursive(home: &Path, targets: &mut Vec<PathBuf>) -> IO_Result<()> {
    let home_iter = read_dir(home)?;
    for entry in home_iter {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            if p.ends_with("target") {
                if hold_docs(&p).is_ok() {
                    targets.push(p);
                }
            } else {
                let _ = walk_dir_recursive(&p, targets)?;
            }
        }
    }
    Ok(())
}

fn hold_docs(path: &Path) -> IO_Result<()> {
    let target_iter = read_dir(path)?;
    for entry in target_iter {
        let e = entry?;
        let ep = e.path();
        if ep.is_dir() && ep.ends_with("doc") {
            return Ok(());
        }
    }
    Err(Error::new(ErrorKind::Other, ""))
}

pub fn find_doc_entries(dir: &Path) -> IO_Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    walk_dir_recursive(dir, &mut results)?;
    Ok(results)
}

fn main() {
    let home_dir = env::home_dir().expect("error get $HOME path");

    if let Some(toolchain_doc_path) = get_toolchain_doc_path() {
        println!("toolchain: {toolchain_doc_path:#?}");
    };

    match find_doc_entries(&home_dir) {
        Ok(v) => println!("find doc paths: {v:#?}"),
        Err(err) => println!("there was an error during walking the dir, {}", err),
    }
}

/*
 * working prototype really slow. Plan to use ignore by BuntSushi to use multithreading for finding
 * my docs
 */
