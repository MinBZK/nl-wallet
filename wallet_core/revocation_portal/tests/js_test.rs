#[cfg(test)]
mod js_tests {
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn test_portal_javascript() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

        if !manifest_dir.join("node_modules").exists() {
            let install = Command::new("npm")
                .args(["ci"])
                .current_dir(manifest_dir)
                .output()
                .expect("Failed to run npm ci");

            assert!(
                install.status.success(),
                "npm ci failed:\n{}",
                String::from_utf8_lossy(&install.stderr)
            );
        }

        let status = Command::new("npm")
            .args(["run", "coverage"])
            .current_dir(manifest_dir)
            .status()
            .expect("Failed to run npm test — is Node.js installed?");

        assert!(status.success(), "JavaScript tests failed");
    }
}
