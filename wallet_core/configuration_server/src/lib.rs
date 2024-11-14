use std::env;
use std::path::PathBuf;

pub mod server;
pub mod settings;

pub fn path_from_root(file_name: &str) -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(file_name)
}

pub fn read_file(file_name: &str) -> Vec<u8> {
    std::fs::read(path_from_root(file_name)).unwrap()
}
