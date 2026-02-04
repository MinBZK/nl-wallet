//! Build script utilities for web applications.
//!
//! Enable with the `build` feature and use as a build-dependency.

use std::path::Path;
use std::process::Command;

/// Copies files and directories recursively to the destination using `cp -R`.
///
/// This function also emits `cargo:rerun-if-changed` directives for each source.
///
/// # Panics
///
/// - On Windows, as it's not supported.
/// - If `cp` fails to execute or returns a non-zero exit code.
pub fn copy_static_assets(sources: &[&Path], dest: &Path) {
    if cfg!(windows) {
        panic!("Building on Windows is not supported");
    }

    for source in sources {
        println!("cargo:rerun-if-changed={}", source.display());

        let status = Command::new("cp")
            .arg("-R")
            .arg(source)
            .arg(dest)
            .status()
            .unwrap_or_else(|e| panic!("Failed to run cp: {}", e));

        if !status.success() {
            panic!("cp -R {} {} failed", source.display(), dest.display());
        }
    }
}
