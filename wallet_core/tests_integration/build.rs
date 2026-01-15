use std::fs;
use std::os;
use std::path::Path;

use serde::de::DeserializeOwned;

use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use utils::path::prefix_local_path;

#[cfg(feature = "performance_test")]
fn inject_dotenv_vars() {
    use std::env;
    use std::path::PathBuf;

    let crate_path: PathBuf = env::var("CARGO_MANIFEST_DIR").expect("Could not get crate path").into();
    let env_file_path = crate_path.join(".env");

    println!("cargo:rerun-if-changed={}", env_file_path.to_str().unwrap());

    match dotenvy::from_path_iter(env_file_path) {
        Ok(values) => {
            for item in values {
                let (key, value) = item.expect("Could not read entry from .env file");
                println!("cargo:rustc-env={key}={value}");
            }
        }
        // Do not panic on this, as we may want to operate without any `.env` file.
        Err(error) => println!("cargo:warning=Could not read .env file: {error}"),
    }
}

fn parse_json<T: DeserializeOwned>(file: &str, fallback: &str) -> T {
    let file_path = prefix_local_path(Path::new(file));
    // If the config file doesn't exist, copy the fallback to the config file and use that
    if !file_path.exists() {
        #[cfg(windows)]
        os::windows::fs::symlink_file(fallback, &file_path).unwrap();

        #[cfg(unix)]
        os::unix::fs::symlink(fallback, &file_path).unwrap();
    }

    let content: T = serde_json::from_slice(&fs::read(&file_path).unwrap()).expect("Could not parse config json");

    println!("cargo:rerun-if-changed={file}");

    content
}

fn main() {
    #[cfg(feature = "performance_test")]
    inject_dotenv_vars();

    let pid_metadata = parse_json::<UncheckedTypeMetadata>("eudi:pid:1.json", "eudi:pid:1-default.json");
    assert_eq!(pid_metadata.vct, "urn:eudi:pid:1");

    let nl_pid_metadata = parse_json::<UncheckedTypeMetadata>("eudi:pid:nl:1.json", "eudi:pid:nl:1-default.json");
    assert_eq!(nl_pid_metadata.vct, "urn:eudi:pid:nl:1");
}
