use std::env;
use std::fs;
use std::path::Path;

use web_utils::build::BuildProfile;

fn main() {
    if cfg!(windows) {
        panic!("Building on Windows is not supported");
    }

    let profile = BuildProfile::from_cargo_profile(env::var("PROFILE").ok().as_deref());
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // Remove stale contents (e.g. broken symlinks from a previous build) before repopulating
    if Path::new("assets/static").exists() {
        fs::remove_dir_all("assets/static").expect("Failed to remove assets/static");
    }
    // Create the necessary directories so that `cp -R` behaves as expected
    fs::create_dir_all("assets/static").expect("Failed to create assets/static");
    // These directories are merged from multiple crates, so they must be copies
    web_utils::build::copy_static_assets(
        &[
            Path::new("../demo_utils/static/images"),
            Path::new("../demo_utils/static/non-free"),
            Path::new("../../lib/web_utils/static/images"),
            Path::new("../../lib/web_utils/static/non-free"),
        ],
        Path::new("assets/static"),
    );

    web_utils::build::link_or_copy_asset(
        Path::new("../../../lib/web_utils/static/language.js"),
        Path::new("assets/language.js"),
        profile,
    );

    // In development mode, symlink CSS directories so changes are reflected immediately
    if !profile.is_release() {
        fs::create_dir_all("assets/demo_utils/static").expect("Failed to create assets/demo_utils/static");
        fs::create_dir_all("assets/lib/web_utils/static").expect("Failed to create assets/lib/web_utils/static");

        // Symlink CSS directories to source
        web_utils::build::force_symlink(Path::new("../../static/css"), Path::new("assets/static/css"));
        web_utils::build::force_symlink(
            Path::new("../../../../demo_utils/static/css"),
            Path::new("assets/demo_utils/static/css"),
        );
        web_utils::build::force_symlink(
            Path::new("../../../../../../lib/web_utils/static/css"),
            Path::new("assets/lib/web_utils/static/css"),
        );

        // Symlink non-free/ and images/ so url() paths resolve to merged assets
        web_utils::build::force_symlink(
            Path::new("../../static/non-free"),
            Path::new("assets/demo_utils/static/non-free"),
        );
        web_utils::build::force_symlink(
            Path::new("../../static/images"),
            Path::new("assets/demo_utils/static/images"),
        );
        web_utils::build::force_symlink(
            Path::new("../../../static/non-free"),
            Path::new("assets/lib/web_utils/static/non-free"),
        );
        web_utils::build::force_symlink(
            Path::new("../../../static/images"),
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
