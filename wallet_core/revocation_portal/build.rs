use std::env;
use std::path::Path;

fn main() {
    web_utils::build::copy_static_assets(
        &[
            Path::new("static/images"),
            Path::new("static/lokalize.js"),
            Path::new("static/portal.js"),
            Path::new("static/portal-ui.js"),
            Path::new("../lib/web_utils/static/images"),
            Path::new("../lib/web_utils/static/non-free"),
            Path::new("../lib/web_utils/static/language.js"),
        ],
        Path::new("assets"),
    );

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("style.css");

    web_utils::build::combine_css(
        &[
            Path::new("../lib/web_utils/static/css/reset.css"),
            Path::new("../lib/web_utils/static/css/button-reset.css"),
            Path::new("../lib/web_utils/static/css/language_selector.css"),
            Path::new("../lib/web_utils/static/css/fonts.css"),
            Path::new("static/css/portal.css"),
        ],
        &dest_path,
    );
}
