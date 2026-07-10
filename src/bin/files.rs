use std::env;
use std::fs::read_dir;
use std::io::Error;
use std::path::PathBuf;
use xdg::{BaseDirectories, BaseDirectoriesError};

fn get_toolchain_doc_path() -> Option<PathBuf> {
    let home = env::var("RUSTUP_HOME").ok()?;
    let toolchain = env::var("RUSTUP_TOOLCHAIN").ok()?;

    let mut path = PathBuf::from(home);
    path.push("toolchains");
    path.push(toolchain);
    path.push("share/doc/rust/html");

    Some(path)
}

fn get_docs_path(home: PathBuf) -> Result<Vec<PathBuf>, Error> {
    let paths = Vec::new();
    let home_iter = read_dir(home)?;
    Ok(paths)
}

fn main() {
    let home_dir = env::home_dir().expect("error get $HOME path");
    let xdg = BaseDirectories::new().expect("error get BaseDirectories");
    let config_home_dir = xdg.get_config_home();

    println!("home dir: {}", home_dir.to_string_lossy());
    println!("config_home: {config_home_dir:#?}");

    if let Some(toolchain_doc_path) = get_toolchain_doc_path() {
        println!("toolchain: {toolchain_doc_path:#?}");
    };
}

/*
 * 1. My tool need to find all target directories in home dir of the user
 * 2.  Check if this directories hold doc directory it must to be added to vector
 * 3. process docs from each directory in the vector
 *
 * Also app need to fined toolchain directory and add set value of variable.
 */
