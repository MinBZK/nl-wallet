use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let is_release = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()) == "release";

    // These directories are merged from multiple crates, so they must be copies
    web_utils::build::copy_static_assets(
        &[
            Path::new("static/non-free"),
            Path::new("../demo_utils/static/images"),
            Path::new("../demo_utils/static/non-free"),
            Path::new("../../lib/web_utils/static/images"),
            Path::new("../../lib/web_utils/static/non-free"),
        ],
        Path::new("assets"),
    );

    // Single-source files can be symlinks for instant dev updates
    force_symlink(Path::new("../static/usecase.js"), Path::new("assets/usecase.js"));
    force_symlink(
        Path::new("../../../lib/web_utils/static/language.js"),
        Path::new("assets/language.js"),
    );

    // In development mode, copy CSS files preserving directory structure so the browser
    // can resolve @import paths. Symlinks for non-free/ and images/ ensure that relative
    // url() references in CSS (e.g. ../non-free/images/logo.svg) resolve to the merged
    // asset directories regardless of which crate the CSS originates from.
    if !is_release {
        fs::create_dir_all("assets/static").expect("Failed to create assets/static");
        fs::create_dir_all("assets/demo_utils/static").expect("Failed to create assets/demo_utils/static");
        fs::create_dir_all("assets/lib/web_utils/static").expect("Failed to create assets/lib/web_utils/static");

        // Symlink CSS directories to source so changes are reflected immediately without rebuild
        force_symlink(Path::new("../../static/css"), Path::new("assets/static/css"));
        force_symlink(Path::new("../../../../demo_utils/static/css"), Path::new("assets/demo_utils/static/css"));
        force_symlink(Path::new("../../../../../../lib/web_utils/static/css"), Path::new("assets/lib/web_utils/static/css"));

        // Symlink non-free/ and images/ so url() paths like ../non-free/images/x.svg resolve to merged assets
        force_symlink(Path::new("../non-free"), Path::new("assets/static/non-free"));
        force_symlink(Path::new("../images"), Path::new("assets/static/images"));
        force_symlink(Path::new("../../non-free"), Path::new("assets/demo_utils/static/non-free"));
        force_symlink(Path::new("../../images"), Path::new("assets/demo_utils/static/images"));
        force_symlink(Path::new("../../../non-free"), Path::new("assets/lib/web_utils/static/non-free"));
        force_symlink(Path::new("../../../images"), Path::new("assets/lib/web_utils/static/images"));
    }

    combine_mijn_amsterdam_css();
    combine_mijn_amsterdam_return_css();
    combine_monkey_bike_css();
    combine_online_marketplace_css();
    combine_xyz_bank_css();
    combine_job_finder_css();
}

fn combine_mijn_amsterdam_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("mijn_amsterdam.css");

    web_utils::build::combine_css_with_imports(
        Path::new("static/css/mijn_amsterdam-index.css"),
        &dest_path,
    );
}

fn combine_mijn_amsterdam_return_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("mijn_amsterdam_return.css");

    web_utils::build::combine_css_with_imports(
        Path::new("static/css/mijn_amsterdam-return.css"),
        &dest_path,
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
    std::os::unix::fs::symlink(target, link)
        .unwrap_or_else(|e| panic!("Failed to create symlink {} -> {}: {}", link.display(), target.display(), e));
}

fn combine_monkey_bike_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("monkey_bike.css");

    web_utils::build::combine_css(
        &[
            Path::new("../../lib/web_utils/static/css/reset.css"),
            Path::new("../../lib/web_utils/static/css/button-reset.css"),
            Path::new("../../lib/web_utils/static/css/fonts.css"),
            Path::new("../../lib/web_utils/static/css/language_selector.css"),
            Path::new("../demo_utils/static/css/demo_bar.css"),
            Path::new("../demo_utils/static/css/common.css"),
            Path::new("../demo_utils/static/css/notification.css"),
            Path::new("../demo_utils/static/css/page.css"),
            Path::new("../demo_utils/static/css/buttons-before.css"),
            Path::new("static/css/monkey_bike.css"),
            Path::new("static/css/title_alt.css"),
            Path::new("static/css/card.css"),
        ],
        &dest_path,
    );
}

fn combine_online_marketplace_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("online_marketplace.css");

    web_utils::build::combine_css(
        &[
            Path::new("../../lib/web_utils/static/css/reset.css"),
            Path::new("../../lib/web_utils/static/css/button-reset.css"),
            Path::new("../../lib/web_utils/static/css/fonts.css"),
            Path::new("../../lib/web_utils/static/css/language_selector.css"),
            Path::new("../demo_utils/static/css/demo_bar.css"),
            Path::new("../demo_utils/static/css/common.css"),
            Path::new("../demo_utils/static/css/notification.css"),
            Path::new("../demo_utils/static/css/buttons-before.css"),
            Path::new("static/css/title_alt.css"),
            Path::new("static/css/card.css"),
            Path::new("static/css/online_marketplace.css"),
        ],
        &dest_path,
    );
}

fn combine_xyz_bank_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("xyz_bank.css");

    web_utils::build::combine_css(
        &[
            Path::new("../../lib/web_utils/static/css/reset.css"),
            Path::new("../../lib/web_utils/static/css/button-reset.css"),
            Path::new("../../lib/web_utils/static/css/fonts.css"),
            Path::new("../../lib/web_utils/static/css/language_selector.css"),
            Path::new("../demo_utils/static/css/demo_bar.css"),
            Path::new("../demo_utils/static/css/common.css"),
            Path::new("../demo_utils/static/css/title_bar.css"),
            Path::new("../demo_utils/static/css/notification.css"),
            Path::new("../demo_utils/static/css/container.css"),
            Path::new("../demo_utils/static/css/buttons-after.css"),
            Path::new("static/css/xyz_bank.css"),
        ],
        &dest_path,
    );
}

fn combine_job_finder_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("job_finder.css");

    web_utils::build::combine_css(
        &[
            Path::new("../../lib/web_utils/static/css/reset.css"),
            Path::new("../../lib/web_utils/static/css/button-reset.css"),
            Path::new("../../lib/web_utils/static/css/fonts.css"),
            Path::new("../../lib/web_utils/static/css/language_selector.css"),
            Path::new("../demo_utils/static/css/demo_bar.css"),
            Path::new("../demo_utils/static/css/common.css"),
            Path::new("../demo_utils/static/css/title_bar.css"),
            Path::new("../demo_utils/static/css/notification.css"),
            Path::new("../demo_utils/static/css/container.css"),
            Path::new("../demo_utils/static/css/buttons-after.css"),
            Path::new("static/css/job_finder.css"),
        ],
        &dest_path,
    );
}
