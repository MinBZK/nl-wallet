use std::fs;
use std::sync::LazyLock;

use base64::prelude::*;
use regex::Regex;

use utils::path::prefix_local_path;

pub mod error;
pub mod headers;
pub mod language;

pub const OPTION_STR_NONE: Option<&str> = None;

pub static LANGUAGE_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(crypto::utils::sha256(include_bytes!("../assets/language.js"))));

pub static WALLET_WEB_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(crypto::utils::sha256(&read_wallet_web())));

pub static WALLET_WEB_CSS_SHA256: LazyLock<String> = LazyLock::new(|| {
    // Same regex as in wallet-web/utils/extract-style-hash.ts
    Regex::new(r#"\[\["styles",\['([^']+)']]]"#)
        .unwrap()
        .captures(&String::from_utf8_lossy(&read_wallet_web()))
        .map(|caps| BASE64_STANDARD.encode(crypto::utils::sha256(caps.get(1).unwrap().as_str().as_bytes())))
        .expect("no style in nl-wallet-web.js found")
});

fn read_wallet_web() -> Vec<u8> {
    fs::read(prefix_local_path("assets/nl-wallet-web.iife.js".as_ref())).expect("no nl-wallet-web.iife.js found")
}
