#[cfg(test)]
mod js_tests {
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn test_portal_javascript() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

        if !manifest_dir.join("node_modules").exists() {
            let install = Command::new("npm")
                .args(["ci", "--ignore-scripts"])
                .current_dir(manifest_dir)
                .output()
                .expect("Failed to run npm ci");

            assert!(
                install.status.success(),
                "npm ci failed:\n{}",
                String::from_utf8_lossy(&install.stderr)
            );
        }

        let output = Command::new("npm")
            .args(["test"])
            .current_dir(manifest_dir)
            .output()
            .expect("Failed to run npm test — is Node.js installed?");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "JavaScript tests failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}
