//! Build script utilities for web applications.
//!
//! Enable with the `build` feature and use as a build-dependency.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Copies files and directories recursively to the destination using `cp -R`.
///
/// This function also emits `cargo::rerun-if-changed` directives for each source.
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
        println!("cargo::rerun-if-changed={}", source.display());

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

/// Combines multiple CSS files into a single file.
///
/// This function also emits `cargo::rerun-if-changed` directives for each source file.
///
/// # Panics
///
/// - If any source file cannot be read.
/// - If the destination file cannot be written.
pub fn combine_css(sources: &[&Path], dest: &Path) {
    let mut combined = String::new();

    for path in sources {
        println!("cargo::rerun-if-changed={}", path.display());

        let content = fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

        combined.push_str(&format!("/* === {} === */\n", file_name));
        combined.push_str(&content);
        combined.push_str("\n\n");
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).expect("Failed to create destination directory");
    }

    fs::write(dest, combined).expect("Failed to write combined CSS");
}
