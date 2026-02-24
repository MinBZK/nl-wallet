use std::sync::OnceLock;

use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use etag::EntityTag;

/// Serves bundled CSS content with ETag-based caching.
///
/// Returns 304 Not Modified if the client's `If-None-Match` header matches the content hash.
pub fn serve_bundled_css(headers: &HeaderMap, css: &'static str) -> Response {
    static BUNDLED_ETAG: OnceLock<EntityTag> = OnceLock::new();
    let bundled_etag = BUNDLED_ETAG.get_or_init(|| EntityTag::from_data(css.as_bytes()));

    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        let Some(entity_tag) = if_none_match.to_str().ok().and_then(|etag| etag.parse().ok()) else {
            return (StatusCode::BAD_REQUEST).into_response();
        };

        if bundled_etag.weak_eq(&entity_tag) {
            return (StatusCode::NOT_MODIFIED, [(header::ETAG, bundled_etag.to_string())]).into_response();
        }
    }

    (
        // We can safely unwrap all header values below because we know there are no non-ascii characters used.
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_str("text/css; charset=utf-8").unwrap(),
            ),
            (header::ETAG, HeaderValue::from_str(&bundled_etag.to_string()).unwrap()),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_str("public, max-age=604800").unwrap(),
            ),
        ],
        css,
    )
        .into_response()
}
