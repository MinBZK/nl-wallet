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
    fs::create_dir_all("assets/static/non-free").expect("Failed to create assets/static/non-free");
    // These directories are merged from multiple crates, so they must be copies
    web_utils::build::copy_static_assets(
        &[
            Path::new("static/non-free"),
            Path::new("../demo_utils/static/images"),
            Path::new("../demo_utils/static/non-free"),
            Path::new("../../lib/web_utils/static/images"),
            Path::new("../../lib/web_utils/static/non-free"),
        ],
        Path::new("assets/static"),
    );

    web_utils::build::link_or_copy_asset(
        Path::new("../static/usecase.js"),
        Path::new("assets/usecase.js"),
        profile,
    );
    web_utils::build::link_or_copy_asset(
        Path::new("../../../lib/web_utils/static/language.js"),
        Path::new("assets/language.js"),
        profile,
    );

    // In development mode, copy CSS files preserving directory structure so the browser
    // can resolve @import paths. Symlinks for non-free/ and images/ ensure that relative
    // url() references in CSS (e.g. ../non-free/images/logo.svg) resolve to the merged
    // asset directories regardless of which crate the CSS originates from.
    if !profile.is_release() {
        fs::create_dir_all("assets/static").expect("Failed to create assets/static");
        fs::create_dir_all("assets/demo_utils/static").expect("Failed to create assets/demo_utils/static");
        fs::create_dir_all("assets/lib/web_utils/static").expect("Failed to create assets/lib/web_utils/static");

        // Symlink CSS directories to source so changes are reflected immediately without rebuild
        web_utils::build::force_symlink(Path::new("../../static/css"), Path::new("assets/static/css"));
        web_utils::build::force_symlink(
            Path::new("../../../../demo_utils/static/css"),
            Path::new("assets/demo_utils/static/css"),
        );
        web_utils::build::force_symlink(
            Path::new("../../../../../../lib/web_utils/static/css"),
            Path::new("assets/lib/web_utils/static/css"),
        );

        // Symlink non-free/ and images/ so url() paths like ../non-free/images/x.svg resolve to merged assets
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

    for entry in [
        "mijn_amsterdam-index",
        "mijn_amsterdam-return",
        "monkey_bike-index",
        "monkey_bike-return",
        "online_marketplace-index",
        "online_marketplace-return",
        "xyz_bank-index",
        "xyz_bank-return",
        "job_finder-index",
        "job_finder-return",
    ] {
        combine_usecase_css(entry, profile, Path::new(&manifest_dir));
    }
}

fn combine_usecase_css(entry_name: &str, profile: BuildProfile, manifest_dir: &Path) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(format!("{entry_name}.css"));

    web_utils::build::combine_css_with_imports(
        &Path::new("static/css").join(format!("{entry_name}.css")),
        &dest_path,
        profile,
        manifest_dir,
    );
}
