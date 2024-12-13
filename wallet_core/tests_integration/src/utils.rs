use std::env;
use std::path::PathBuf;

use wallet_common::urls::BaseUrl;

pub(crate) fn path_from_root(file_name: &str) -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(file_name)
}

pub(crate) fn read_file(file_name: &str) -> Vec<u8> {
    std::fs::read(path_from_root(file_name)).unwrap()
}

pub(crate) fn remove_path(base_url: &BaseUrl) -> BaseUrl {
    let mut base_url = base_url.as_ref().clone();
    base_url.set_path("");
    base_url.try_into().unwrap()
}
