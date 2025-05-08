use axum::extract::Request;
use axum::http::header::CACHE_CONTROL;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

use http_utils::urls::CorsOrigin;

pub fn cors_layer(allow_origins: CorsOrigin) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(allow_origins)
        .allow_headers(Any)
        .allow_methods([Method::GET, Method::POST])
}

pub async fn set_static_cache_control(request: Request, next: Next) -> Response {
    // only cache images and fonts, not CSS and JS (except wallet_web, as that is suffixed with a hash)
    let set_no_store = !request.uri().path().ends_with(".iife.js")
        && [".css", ".js"].iter().any(|ext| request.uri().path().ends_with(ext));
    let mut response = next.run(request).await;
    if set_no_store {
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}
