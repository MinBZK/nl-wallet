use std::{env, fs, path::PathBuf};

fn manifest_path() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap()
}

pub fn read_file(name: &str) -> String {
    fs::read_to_string(manifest_path().join(format!("tests/resources/{}", name))).unwrap()
}
