use std::{env, path::PathBuf};

pub mod server;
pub mod settings;

pub fn read_config_jwt() -> Vec<u8> {
    let root_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();
    let config_file = root_path.join("wallet-config-jws-compact.txt");
    std::fs::read(config_file.as_path()).unwrap()
}
