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

fn write_to_file(dest: &Path, content: &str) {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).expect("Failed to create destination directory");
    }
    fs::write(dest, content).expect("Failed to write CSS");
}

/// Combines CSS files with @import resolution using lightningcss.
///
/// This function processes CSS files differently based on the build profile:
/// - **Release builds**: Resolves all @import statements and minifies the result into a single file
/// - **Development builds**: Writes a placeholder so `include_str!` compiles. CSS is served directly from the
///   filesystem at runtime, with the browser resolving @import statements.
///
/// # Arguments
///
/// * `entry_file` - The main CSS file that may contain @import statements
/// * `dest` - Destination path for the combined CSS file
///
/// # Panics
///
/// - If any source file cannot be read (release mode only)
/// - If the destination file cannot be written
/// - If CSS parsing or bundling fails (release mode only)
pub fn combine_css_with_imports(entry_file: &Path, dest: &Path) {
    let is_release = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()) == "release";

    println!("cargo::rerun-if-changed={}", entry_file.display());

    if !is_release {
        write_to_file(dest, "/* CSS served from filesystem in development mode */");
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let abs_entry_file = Path::new(&manifest_dir).join(entry_file);

    // Use the bundler to resolve @import statements
    let fs = lightningcss::bundler::FileProvider::new();
    let mut bundler =
        lightningcss::bundler::Bundler::new(&fs, None, lightningcss::stylesheet::ParserOptions::default());

    let mut stylesheet = bundler
        .bundle(&abs_entry_file)
        .unwrap_or_else(|e| panic!("Failed to bundle CSS {}: {:?}", entry_file.display(), e));

    stylesheet
        .minify(lightningcss::stylesheet::MinifyOptions::default())
        .ok();

    let res = stylesheet
        .to_css(lightningcss::stylesheet::PrinterOptions {
            minify: true,
            ..lightningcss::stylesheet::PrinterOptions::default()
        })
        .unwrap_or_else(|e| panic!("Failed to generate CSS: {:?}", e));

    write_to_file(dest, &res.code);
}
