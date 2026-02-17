use std::env;
use std::fs;
use std::path::Path;

use web_utils::build::BuildProfile;

fn main() {
    let profile = BuildProfile::from_cargo_profile(env::var("PROFILE").ok().as_deref());
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // These directories are merged from multiple crates, so they must be copies
    web_utils::build::copy_static_assets(
        &[
            Path::new("../demo_utils/static/images"),
            Path::new("../demo_utils/static/non-free"),
            Path::new("../../lib/web_utils/static/images"),
            Path::new("../../lib/web_utils/static/non-free"),
        ],
        Path::new("assets"),
    );

    // Single-source files can be symlinks for instant dev updates
    force_symlink(
        Path::new("../../../lib/web_utils/static/language.js"),
        Path::new("assets/language.js"),
    );

    // In development mode, symlink CSS directories so changes are reflected immediately
    if !profile.is_release() {
        fs::create_dir_all("assets/static").expect("Failed to create assets/static");
        fs::create_dir_all("assets/demo_utils/static").expect("Failed to create assets/demo_utils/static");
        fs::create_dir_all("assets/lib/web_utils/static").expect("Failed to create assets/lib/web_utils/static");

        // Symlink CSS directories to source
        force_symlink(Path::new("../../static/css"), Path::new("assets/static/css"));
        force_symlink(
            Path::new("../../../../demo_utils/static/css"),
            Path::new("assets/demo_utils/static/css"),
        );
        force_symlink(
            Path::new("../../../../../../lib/web_utils/static/css"),
            Path::new("assets/lib/web_utils/static/css"),
        );

        // Symlink non-free/ and images/ so url() paths resolve to merged assets
        force_symlink(Path::new("../non-free"), Path::new("assets/static/non-free"));
        force_symlink(Path::new("../images"), Path::new("assets/static/images"));
        force_symlink(
            Path::new("../../non-free"),
            Path::new("assets/demo_utils/static/non-free"),
        );
        force_symlink(Path::new("../../images"), Path::new("assets/demo_utils/static/images"));
        force_symlink(
            Path::new("../../../non-free"),
            Path::new("assets/lib/web_utils/static/non-free"),
        );
        force_symlink(
            Path::new("../../../images"),
            Path::new("assets/lib/web_utils/static/images"),
        );
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("style.css");

    web_utils::build::combine_css_with_imports(
        Path::new("static/css/nav.css"),
        &dest_path,
        profile,
        Path::new(&manifest_dir),
    );
}

/// Creates a symlink, removing any existing file, symlink, or directory at the link path.
fn force_symlink(target: &Path, link: &Path) {
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
