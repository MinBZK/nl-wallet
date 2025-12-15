use std::borrow::Cow;
use std::env;
use std::path::Path;
use std::path::PathBuf;

/// If the file path is relative and this binary is ran through cargo,
/// prepend the directory that contains the `Cargo.toml` to the path.
/// Otherwise return the file path unchanged.
pub fn prefix_local_path<'a>(file_path: impl Into<Cow<'a, Path>>) -> Cow<'a, Path> {
    let file_path = file_path.into();
    if file_path.is_relative()
        && let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR")
    {
        Cow::Owned(PathBuf::from(manifest_dir).join(file_path))
    } else {
        file_path
    }
}
