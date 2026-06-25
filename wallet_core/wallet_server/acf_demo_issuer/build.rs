use std::env;
use std::fs;
use std::path::Path;

use web_utils::build::BuildProfile;
use web_utils::build::combine_css_with_imports;
use web_utils::build::force_symlink;

fn main() {
    if cfg!(windows) {
        panic!("Building on Windows is not supported");
    }

    let profile = BuildProfile::from_cargo_profile(env::var("PROFILE").ok().as_deref());
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    if Path::new("assets/static").exists() {
        fs::remove_dir_all("assets/static").expect("Failed to remove assets/static");
    }
    fs::create_dir_all("assets/static").expect("Failed to create assets/static");

    // Copy images and non-free assets (fonts, icons) from demo_utils and web_utils into assets/static/
    web_utils::build::copy_static_assets(
        &[
            Path::new("../../demo/demo_utils/static/images"),
            Path::new("../../demo/demo_utils/static/non-free"),
            Path::new("../../lib/web_utils/static/images"),
            Path::new("../../lib/web_utils/static/non-free"),
        ],
        Path::new("assets/static"),
    );

    if !profile.is_release() {
        fs::create_dir_all("assets/demo/demo_utils/static").expect("Failed to create assets/demo/demo_utils/static");
        fs::create_dir_all("assets/lib/web_utils/static").expect("Failed to create assets/lib/web_utils/static");

        // Own CSS — symlinked so edits are reflected immediately
        force_symlink(Path::new("../../static/css"), Path::new("assets/static/css"));

        // Cross-crate CSS — browser fetches these via @import from consent.css
        force_symlink(
            Path::new("../../../../../../demo/demo_utils/static/css"),
            Path::new("assets/demo/demo_utils/static/css"),
        );
        force_symlink(
            Path::new("../../../../../../lib/web_utils/static/css"),
            Path::new("assets/lib/web_utils/static/css"),
        );

        // Redirect image/non-free url() references inside those CSS files back to assets/static/
        force_symlink(
            Path::new("../../../static/non-free"),
            Path::new("assets/demo/demo_utils/static/non-free"),
        );
        force_symlink(
            Path::new("../../../static/images"),
            Path::new("assets/demo/demo_utils/static/images"),
        );
        force_symlink(
            Path::new("../../../static/non-free"),
            Path::new("assets/lib/web_utils/static/non-free"),
        );
        force_symlink(
            Path::new("../../../static/images"),
            Path::new("assets/lib/web_utils/static/images"),
        );
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("consent.css");

    combine_css_with_imports(
        Path::new("static/css/consent.css"),
        &dest_path,
        profile,
        Path::new(&manifest_dir),
    );
}
