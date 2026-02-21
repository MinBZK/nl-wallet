//! Build script utilities for web applications.
//!
//! Enable with the `build` feature and use as a build-dependency.

use std::fs;
use std::path::Path;
use std::process::Command;

/// The build profile as determined by Cargo's `PROFILE` environment variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    /// Converts the value of Cargo's `PROFILE` environment variable to a `BuildProfile`.
    pub fn from_cargo_profile(profile: Option<&str>) -> Self {
        match profile {
            Some("release") => Self::Release,
            _ => Self::Debug,
        }
    }

    pub fn is_release(self) -> bool {
        self == Self::Release
    }
}

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

/// Creates a symlink, removing any existing file, symlink, or directory at the link path.
///
/// # Panics
///
/// - If the existing file/symlink/directory at `link` cannot be removed
/// - If the symlink creation fails
pub fn force_symlink(target: &Path, link: &Path) {
    if let Ok(meta) = link.symlink_metadata() {
        if meta.is_dir() {
            fs::remove_dir_all(link).unwrap_or_else(|e| panic!("Failed to remove dir {}: {}", link.display(), e));
        } else {
            fs::remove_file(link).unwrap_or_else(|e| panic!("Failed to remove {}: {}", link.display(), e));
        }
    }
    std::os::unix::fs::symlink(target, link).unwrap_or_else(|e| {
        panic!(
            "Failed to create symlink {} -> {}: {}",
            link.display(),
            target.display(),
            e
        )
    });
}

/// Makes a static asset available at `link`, using the strategy appropriate for the build profile:
/// - **Debug**: Creates a symlink for instant dev updates without rebuild
/// - **Release**: Copies the file for Docker compatibility (symlinks break in containers)
///
/// The `target` path is relative to the `link` location (as with symlinks). For copying, the actual
/// source path is resolved by joining `target` onto the parent directory of `link`.
///
/// # Panics
///
/// - If the existing file/symlink/directory at `link` cannot be removed
/// - If the symlink or copy operation fails
pub fn link_or_copy_asset(target: &Path, link: &Path, profile: BuildProfile) {
    if profile.is_release() {
        let source = link.parent().unwrap().join(target);
        // Remove existing file/symlink before copying
        if link.symlink_metadata().is_ok() {
            fs::remove_file(link).unwrap_or_else(|e| panic!("Failed to remove {}: {}", link.display(), e));
        }
        fs::copy(&source, link)
            .unwrap_or_else(|e| panic!("Failed to copy {} to {}: {}", source.display(), link.display(), e));
    } else {
        force_symlink(target, link);
    }
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
/// * `entry_file` - The main CSS file (relative to the crate root) that may contain @import statements
/// * `dest` - Destination path for the combined CSS file
/// * `profile` - The build profile (debug or release)
/// * `manifest_dir` - The crate's manifest directory (CARGO_MANIFEST_DIR)
///
/// # Panics
///
/// - If any source file cannot be read (release mode only)
/// - If the destination file cannot be written
/// - If CSS parsing or bundling fails (release mode only)
pub fn combine_css_with_imports(entry_file: &Path, dest: &Path, profile: BuildProfile, manifest_dir: &Path) {
    println!("cargo::rerun-if-changed={}", entry_file.display());

    if !profile.is_release() {
        write_to_file(dest, "/* CSS served from filesystem in development mode */");
        return;
    }

    let abs_entry_file = manifest_dir.join(entry_file);

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
