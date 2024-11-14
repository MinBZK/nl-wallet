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
                println!("cargo:rustc-env={}={}", key, value);
            }
        }
        // Do not panic on this, as we may want to operate without any `.env` file.
        Err(error) => println!("cargo:warning=Could not read .env file: {}", error),
    }
}

fn main() {
    #[cfg(feature = "performance_test")]
    inject_dotenv_vars();
}
