use http_utils::urls::BaseUrl;
use utils::path::prefix_local_path;

pub fn read_file(file_name: &str) -> Vec<u8> {
    std::fs::read(prefix_local_path(file_name.as_ref()).as_ref()).unwrap()
}

pub(crate) fn remove_path(base_url: &BaseUrl) -> BaseUrl {
    let mut base_url = base_url.as_ref().clone();
    base_url.set_path("");
    base_url.try_into().unwrap()
}
