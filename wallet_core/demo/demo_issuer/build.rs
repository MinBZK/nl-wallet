use std::env;
use std::path::Path;

fn main() {
    web_utils::build::copy_static_assets(
        &[
            Path::new("static/images"),
            Path::new("../demo_utils/static/images"),
            Path::new("../demo_utils/static/non-free"),
            Path::new("../../lib/web_utils/static/images"),
            Path::new("../../lib/web_utils/static/non-free"),
            Path::new("../../lib/web_utils/static/language.js"),
        ],
        Path::new("assets"),
    );

    combine_housing_css();
    combine_insurance_css();
    combine_university_css();
}

fn combine_housing_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("housing.css");

    web_utils::build::combine_css(
        &[
            Path::new("../../lib/web_utils/static/css/button-reset.css"),
            Path::new("../../lib/web_utils/static/css/fonts.css"),
            Path::new("../../lib/web_utils/static/css/language_selector.css"),
            Path::new("../../lib/web_utils/static/css/reset.css"),
            Path::new("../demo_utils/static/css/demo_bar.css"),
            Path::new("../demo_utils/static/css/common.css"),
            Path::new("../demo_utils/static/css/title_bar.css"),
            Path::new("../demo_utils/static/css/notification.css"),
            Path::new("../demo_utils/static/css/page.css"),
            Path::new("../demo_utils/static/css/buttons-after.css"),
            Path::new("static/css/banner.css"),
            Path::new("static/css/housing.css"),
        ],
        &dest_path,
    );
}

fn combine_insurance_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("insurance.css");

    web_utils::build::combine_css(
        &[
            Path::new("../../lib/web_utils/static/css/button-reset.css"),
            Path::new("../../lib/web_utils/static/css/fonts.css"),
            Path::new("../../lib/web_utils/static/css/language_selector.css"),
            Path::new("../../lib/web_utils/static/css/reset.css"),
            Path::new("../demo_utils/static/css/demo_bar.css"),
            Path::new("../demo_utils/static/css/common.css"),
            Path::new("../demo_utils/static/css/title_bar.css"),
            Path::new("../demo_utils/static/css/notification.css"),
            Path::new("../demo_utils/static/css/page.css"),
            Path::new("../demo_utils/static/css/buttons-after.css"),
            Path::new("static/css/banner.css"),
            Path::new("static/css/insurance.css"),
        ],
        &dest_path,
    );
}

fn combine_university_css() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("university.css");

    web_utils::build::combine_css(
        &[
            Path::new("../../lib/web_utils/static/css/button-reset.css"),
            Path::new("../../lib/web_utils/static/css/fonts.css"),
            Path::new("../../lib/web_utils/static/css/language_selector.css"),
            Path::new("../../lib/web_utils/static/css/reset.css"),
            Path::new("../demo_utils/static/css/demo_bar.css"),
            Path::new("../demo_utils/static/css/common.css"),
            Path::new("../demo_utils/static/css/title_bar.css"),
            Path::new("../demo_utils/static/css/notification.css"),
            Path::new("../demo_utils/static/css/container.css"),
            Path::new("../demo_utils/static/css/buttons-after.css"),
            Path::new("static/css/banner.css"),
            Path::new("static/css/university.css"),
        ],
        &dest_path,
    );
}
