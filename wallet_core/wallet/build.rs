use std::env;
use std::fs;
use std::os;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

use wallet_common::config::config_server_config::ConfigServerConfiguration;
use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::config::EnvironmentSpecific;

fn parse_and_verify_json<T: DeserializeOwned + EnvironmentSpecific>(file: &str, fallback: &str) {
    let crate_path: PathBuf = env::var("CARGO_MANIFEST_DIR").expect("Could not get crate path").into();
    let file_path = crate_path.join(file);
    // If the config file doesn't exist, copy the fallback to the config file and use that
    if !file_path.exists() {
        #[cfg(windows)]
        os::windows::fs::symlink_file(fallback, &file_path).unwrap();

        #[cfg(unix)]
        os::unix::fs::symlink(fallback, &file_path).unwrap();
    }

    let config: T = serde_json::from_slice(&fs::read(file_path).unwrap()).expect("Could not parse config json");

    verify_environment(config.environment(), file);

    println!("cargo:rerun-if-changed={}", file);
}

fn current_env() -> String {
    let env = env::var("CONFIG_ENV");
    let profile = env::var("PROFILE").unwrap();
    if profile == "release" {
        env.expect("CONFIG_ENV environment variable should be set for releases")
    } else {
        env.unwrap_or(String::from("dev"))
    }
}

fn verify_environment(config_env: &str, file: &str) {
    if config_env != current_env() {
        panic!(
            "Build environment '{}' doesn't match config enviroment '{}' for {}",
            current_env(),
            config_env,
            file,
        );
    }
}

fn verify_configurations() {
    parse_and_verify_json::<WalletConfiguration>("wallet-config.json", "default-wallet-config.json");
    parse_and_verify_json::<ConfigServerConfiguration>(
        "config-server-config.json",
        "default-config-server-config.json",
    );
    println!("cargo::rerun-if-env-changed=CONFIG_ENV");
    println!("cargo::rerun-if-env-changed=PROFILE");
}

fn main() {
    verify_configurations();
}
