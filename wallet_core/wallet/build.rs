#[cfg(feature = "env_config")]
use std::{env, path::PathBuf};

#[cfg(feature = "env_config")]
fn inject_dotenv_vars() {
    let crate_path: PathBuf = env::var("CARGO_MANIFEST_DIR").expect("Could not get crate path").into();
    let env_file_path = crate_path.join(".env");

    println!("cargo:rerun-if-changed={}", env_file_path.to_str().unwrap());

    match dotenvy::from_path_iter(env_file_path) {
        Ok(values) => {
            for item in values {
                let (key, value) = item.expect("Could not read entry from .env file");
                println!("cargo:rustc-env={}={}", key, value)
            }
        }
        // Do not panic on this, as we may want to operate without any `.env` file.
        Err(error) => println!("cargo:warning=Could not read .env file: {}", error),
    }
}

fn main() {
    #[cfg(feature = "env_config")]
    inject_dotenv_vars();
}
