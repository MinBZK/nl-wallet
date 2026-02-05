use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;

/// Serve CSS content with ETag and cache control headers.
///
/// # Arguments
///
/// * `headers` - Request headers to check for `If-None-Match`.
/// * `content` - The CSS content to serve.
/// * `sha256_hash` - The base64-encoded SHA256 hash of the CSS content, used as the ETag.
pub fn serve_css(headers: &HeaderMap, content: &'static str, sha256_hash: &str) -> Response {
    let etag = format!("\"{}\"", sha256_hash);

    // Check If-None-Match header for conditional request
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH)
        && if_none_match.as_bytes() == etag.as_bytes()
    {
        return (StatusCode::NOT_MODIFIED, [(header::ETAG, etag)]).into_response();
    }

    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8".to_string()),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable".to_string()),
        ],
        content,
    )
        .into_response()
}
