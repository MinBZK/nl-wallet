use std::{env, fs, path::PathBuf};

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

#[cfg(feature = "env_config")]
fn inject_dotenv_vars() {
    let crate_path: PathBuf = env::var("CARGO_MANIFEST_DIR").expect("Could not get crate path").into();
    let env_file_path = crate_path.join(".env");

    println!("cargo:rerun-if-changed={}", env_file_path.to_str().unwrap());

    match dotenvy::from_path_iter(env_file_path) {
        Ok(values) => {
            for item in values {
                let (key, value) = item.expect("Could not read entry from .env file");
                println!("cargo:rustc-env={}={}", key, value);
            }
        }
        // Do not panic on this, as we may want to operate without any `.env` file.
        Err(error) => println!("cargo:warning=Could not read .env file: {}", error),
    }
}

fn main() {
    android_x86_64_workaround();

    #[cfg(feature = "env_config")]
    inject_dotenv_vars();
}
