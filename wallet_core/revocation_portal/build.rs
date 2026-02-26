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

    fs::create_dir_all("assets/support").expect("Failed to create assets/support");

    web_utils::build::link_or_copy_asset(
        Path::new("../../static/lokalize.js"),
        Path::new("assets/support/lokalize.js"),
        profile,
    );
    web_utils::build::link_or_copy_asset(
        Path::new("../../static/portal.js"),
        Path::new("assets/support/portal.js"),
        profile,
    );
    web_utils::build::link_or_copy_asset(
        Path::new("../../static/portal-ui.js"),
        Path::new("assets/support/portal-ui.js"),
        profile,
    );
    web_utils::build::link_or_copy_asset(
        Path::new("../../../lib/web_utils/static/language.js"),
        Path::new("assets/support/language.js"),
        profile,
    );

    if profile.is_release() {
        fs::create_dir_all("assets/support/static").expect("Failed to create assets/support/static");

        web_utils::build::copy_static_assets(&[Path::new("static/images")], Path::new("assets/support"));
        web_utils::build::copy_static_assets(
            &[
                Path::new("../lib/web_utils/static/images"),
                Path::new("../lib/web_utils/static/non-free"),
            ],
            Path::new("assets/support/static"),
        );
    } else {
        // In development mode, symlink CSS directories so changes are reflected immediately

        fs::create_dir_all("assets/support/static/css").expect("Failed to create assets/static");
        fs::create_dir_all("assets/support/lib/web_utils/static")
            .expect("Failed to create assets/lib/web_utils/static");

        // Symlink CSS directories to source
        web_utils::build::force_symlink(
            Path::new("../../../../static/portal.css"),
            Path::new("assets/support/static/css/portal.css"),
        );
        web_utils::build::force_symlink(
            Path::new("../../../../../../lib/web_utils/static/css"),
            Path::new("assets/support/lib/web_utils/static/css"),
        );

        // Symlink non-free/ and images/ so url() paths resolve to merged assets
        web_utils::build::force_symlink(Path::new("../../static/images"), Path::new("assets/support/images"));
        web_utils::build::force_symlink(
            Path::new("../../../../lib/web_utils/static/non-free"),
            Path::new("assets/support/static/non-free"),
        );
        web_utils::build::force_symlink(
            Path::new("../../../../lib/web_utils/static/images"),
            Path::new("assets/support/static/images"),
        );
        web_utils::build::force_symlink(
            Path::new("../../../static/non-free"),
            Path::new("assets/support/lib/web_utils/static/non-free"),
        );
        web_utils::build::force_symlink(
            Path::new("../../../static/images"),
            Path::new("assets/support/lib/web_utils/static/images"),
        );
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("style.css");

    web_utils::build::combine_css_with_imports(
        Path::new("static/portal.css"),
        &dest_path,
        profile,
        Path::new(&manifest_dir),
    );
}
