use std::env;
use std::path::Path;

fn main() {
    web_utils::build::copy_static_assets(
        &[
            Path::new("static/non-free"),
            Path::new("static/usecase.js"),
            Path::new("../demo_utils/static/images"),
            Path::new("../demo_utils/static/non-free"),
            Path::new("../../lib/web_utils/static/images"),
            Path::new("../../lib/web_utils/static/non-free"),
            Path::new("../../lib/web_utils/static/language.js"),
        ],
        Path::new("assets"),
    );

    combine_mijn_amsterdam_css();
    combine_monkey_bike_css();
    combine_online_marketplace_css();
    combine_xyz_bank_css();
    combine_job_finder_css();
}

fn combine_mijn_amsterdam_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("mijn_amsterdam.css");

    web_utils::build::combine_css(
        &[
            Path::new("../../lib/web_utils/static/css/reset.css"),
            Path::new("../../lib/web_utils/static/css/button-reset.css"),
            Path::new("../../lib/web_utils/static/css/fonts.css"),
            Path::new("../../lib/web_utils/static/css/language_selector.css"),
            Path::new("../demo_utils/static/css/demo_bar.css"),
            Path::new("../demo_utils/static/css/common.css"),
            Path::new("../demo_utils/static/css/title_bar.css"),
            Path::new("../demo_utils/static/css/container.css"),
            Path::new("../demo_utils/static/css/notification.css"),
            Path::new("../demo_utils/static/css/buttons-after.css"),
            Path::new("static/css/mijn_amsterdam.css"),
        ],
        &dest_path,
    );
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
