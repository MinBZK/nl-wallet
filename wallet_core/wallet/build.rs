use std::env;
use std::fs;
use std::os;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;
use wallet_configuration::EnvironmentSpecific;
use wallet_common::utils;

/// Add a temporary workaround for compiling for the Android x86_64 target, which is missing a symbol required by
/// "sqlite3-sys". The root cause of the issue is documented here: https://github.com/rust-lang/rust/issues/109717.
///
/// This is inspired by the following code:
/// * https://github.com/mozilla/application-services/pull/5442/commits/2c97beb435e812f8ffd3f777ad056e6934b97ecc
/// * https://github.com/matrix-org/matrix-rust-sdk/pull/1782/files
fn android_x86_64_workaround() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS environment variable not set.");
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH environment variable not set.");

    // Only imlement workaround if we are compiling for android x86_64.
    if target_os == "android" && target_arch == "x86_64" {
        // The host OS is used in the search path below.
        let host_os = match env::consts::OS {
            "macos" => "darwin",
            os => os,
        };

        // cargo-ndk figures out the path to the NDK for us, we just need to strip off
        // "../build/cmake/android.toolchain.cmake", then add the path to clang for the target architecture.
        let toolchain_path = PathBuf::from(
            env::var("CARGO_NDK_CMAKE_TOOLCHAIN_PATH").expect("CARGO_CFG_TARGET_ARCH environment variable not set."),
        );
        let linux_x86_64_clang_dir = toolchain_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("toolchains")
            .join("llvm")
            .join("prebuilt")
            .join(format!("{host_os}-x86_64"))
            .join("lib")
            .join("clang");

        // We need to find the correct clang version directory to add (e.g. "17.0.2"),
        // so in order to find the last available version, sort the subdirectories
        // and pick the last one.
        let mut linux_x86_64_lib_dir_subdirs = fs::read_dir(&linux_x86_64_clang_dir)
            .unwrap_or_else(|_| panic!("Could not read directory: {}", linux_x86_64_clang_dir.to_str().unwrap()))
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect::<Vec<_>>();
        linux_x86_64_lib_dir_subdirs.sort();

        let linux_x86_64_lib_dir = linux_x86_64_lib_dir_subdirs
            .last()
            .unwrap_or_else(|| {
                panic!(
                    "Could not find subdirectory in path: {}",
                    linux_x86_64_clang_dir.to_str().unwrap()
                )
            })
            .join("lib")
            .join("linux");

        if !(linux_x86_64_lib_dir.exists() && linux_x86_64_lib_dir.is_dir()) {
            panic!("Could not find directory: {}", linux_x86_64_lib_dir.to_str().unwrap())
        }

        // Inform rustc that we need to link "libclang_rt.builtins-x86_64-android.a"
        // and add the path derived above to the linker search paths.
        println!("cargo:rustc-link-search={}", linux_x86_64_lib_dir.to_str().unwrap());
        println!("cargo:rustc-link-lib=static=clang_rt.builtins-x86_64-android");
    }
}

fn parse_and_verify_json<T: DeserializeOwned + EnvironmentSpecific>(file: &str, fallback: &str) {
    let file_path = utils::prefix_local_path(file.as_ref());
    // If the config file doesn't exist, copy the fallback to the config file and use that
    if !file_path.exists() {
        #[cfg(windows)]
        os::windows::fs::symlink_file(fallback, &file_path).unwrap();

        #[cfg(unix)]
        os::unix::fs::symlink(fallback, &file_path).unwrap();
    }

    let config: T = serde_json::from_slice(&fs::read(&file_path).unwrap()).expect("Could not parse config json");

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
    android_x86_64_workaround();

    verify_configurations();
}
