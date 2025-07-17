#![allow(
    clippy::allow_attributes,
    reason = "This file includes generated code that uses `allow` attributes."
)]
use std::env;

// The file has been placed there by the build script.
include!(concat!(env!("OUT_DIR"), "/built.rs"));

pub fn version_string() -> String {
    let dirty_flag: &str = if GIT_DIRTY.unwrap_or_default() {
        "+modifications"
    } else {
        ""
    };
    format!(
        "{} ({}/{}, {}-mode, built: {}, commit: {}{})",
        PKG_VERSION,
        CFG_OS,
        CFG_TARGET_ARCH,
        PROFILE,
        BUILT_TIME_UTC,
        GIT_COMMIT_HASH_SHORT.unwrap_or("n/a"),
        dirty_flag,
    )
}
