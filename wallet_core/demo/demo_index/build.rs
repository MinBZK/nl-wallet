use std::path::Path;

fn main() {
    web_utils::build::copy_static_assets(
        &[
            Path::new("static/css"),
            Path::new("../demo_utils/static/"),
            Path::new("../../lib/web_utils/static/"),
        ],
        Path::new("assets"),
    );
}
