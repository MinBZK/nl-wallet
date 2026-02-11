use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;

use crypto::utils::sha256;

/// Serves bundled CSS content with ETag-based caching.
///
/// Returns 304 Not Modified if the client's `If-None-Match` header matches the content hash.
pub fn serve_bundled_css(headers: &HeaderMap, css: &'static str) -> Response {
    let etag = format!("\"{}\"", BASE64_STANDARD.encode(sha256(css.as_bytes())));

    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH)
        && if_none_match.as_bytes() == etag.as_bytes()
    {
        return (StatusCode::NOT_MODIFIED, [(header::ETAG, etag)]).into_response();
    }

    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8".to_string()),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, "public, max-age=604800".to_string()),
        ],
        css,
    )
        .into_response()
}
