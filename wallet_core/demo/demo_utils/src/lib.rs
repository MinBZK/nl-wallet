use std::fs;
use std::sync::LazyLock;

use base64::prelude::*;

use utils::path::prefix_local_path;

pub mod error;
pub mod headers;
pub mod language;

pub const OPTION_STR_NONE: Option<&str> = None;

pub static LANGUAGE_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(crypto::utils::sha256(include_bytes!("../assets/language.js"))));

pub static WALLET_WEB_JS_SHA256: LazyLock<String> = LazyLock::new(|| {
    fs::read(prefix_local_path("../demo_utils/assets/nl-wallet-web.iife.js".as_ref()))
        .map(|f| BASE64_STANDARD.encode(crypto::utils::sha256(&f)))
        .unwrap_or_default()
});
pub static WALLET_WEB_CSS_SHA256: LazyLock<String> = LazyLock::new(|| {
    fs::read(prefix_local_path("../demo_utils/assets/nl-wallet-web.css".as_ref()))
        .map(|f| BASE64_STANDARD.encode(crypto::utils::sha256(&f)))
        .unwrap_or_default()
});
