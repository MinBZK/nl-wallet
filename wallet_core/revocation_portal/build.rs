use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Base path for shared CSS (web_utils)
    let web_utils_css = Path::new("../lib/web_utils/assets/css");

    // Local CSS
    let local_css = Path::new("assets/css");

    let css_sources: Vec<(&Path, &str)> = vec![
        (web_utils_css, "reset.css"),
        (web_utils_css, "button-reset.css"),
        (web_utils_css, "language_selector.css"),
        (web_utils_css, "fonts.css"),
        (local_css, "portal.css"),
    ];

    let mut combined = String::new();
    for (base_path, file) in &css_sources {
        let path = base_path.join(file);
        println!("cargo:rerun-if-changed={}", path.display());

        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

        combined.push_str(&format!("/* === {} === */\n", file));
        combined.push_str(&content);
        combined.push_str("\n\n");
    }

    let dest_path = Path::new(&out_dir).join("style.css");
    fs::write(&dest_path, &combined).expect("Failed to write combined CSS");
}
