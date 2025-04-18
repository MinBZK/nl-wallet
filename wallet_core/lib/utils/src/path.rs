use std::borrow::Cow;
use std::env;
use std::path::Path;
use std::path::PathBuf;

/// If the file path is relative and this binary is ran through cargo,
/// prepend the directory that contains the `Cargo.toml` to the path.
/// Otherwise return the file path unchanged.
pub fn prefix_local_path(file_path: &Path) -> Cow<'_, Path> {
    let dev_path = file_path
        .is_relative()
        .then(|| {
            env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|base_path| PathBuf::from(base_path).join(file_path))
        })
        .flatten();

    match dev_path {
        Some(dev_path) => Cow::Owned(dev_path),
        None => Cow::Borrowed(file_path),
    }
}
