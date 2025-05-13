use std::sync::LazyLock;

use base64::prelude::*;

pub mod error;
pub mod headers;
pub mod language;

pub const OPTION_STR_NONE: Option<&str> = None;

pub static LANGUAGE_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(crypto::utils::sha256(include_bytes!("../assets/language.js"))));
