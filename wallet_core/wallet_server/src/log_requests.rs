use axum::body::Body;
use axum::body::Bytes;
use axum::body::{self};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use base64::prelude::*;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use http::StatusCode;
use http::Uri;
use http::Version;

use openid4vc::disclosure_session::APPLICATION_OAUTH_AUTHZ_REQ_JWT;
use wallet_common::http_error::APPLICATION_PROBLEM_JSON;

const MAX_BODY_SIZE: usize = 8 * 1024 * 1024; // 8 MiB

pub(crate) async fn log_request_response(req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = log_request(body, &parts.method, &parts.uri, &parts.headers, &parts.version).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = log_response(body, parts.status, &parts.headers, &parts.version).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn body_to_bytes(body: Body) -> Result<Bytes, (StatusCode, String)> {
    body::to_bytes(body, MAX_BODY_SIZE)
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("failed to read body: {}", err)))
}

fn print_headers(headers: &HeaderMap<HeaderValue>) -> String {
    headers
        .iter()
        .map(|(key, val)| format!("{}: {}", key, val.to_str().unwrap_or("[non-ASCII HTTP header value]")))
        .collect::<Vec<_>>()
        .join("\n")
}

fn body_to_string(bytes: &Bytes, headers: &HeaderMap<HeaderValue>) -> String {
    if bytes.is_empty() {
        return "".to_string();
    }

    let content_type = headers
        .get("content-type")
        .map(|header| header.as_bytes())
        .unwrap_or_default();

    if [
        "application/json",
        "application/x-www-form-urlencoded",
        "text/plain",
        APPLICATION_OAUTH_AUTHZ_REQ_JWT.as_ref(),
        APPLICATION_PROBLEM_JSON.as_ref(),
    ]
    .into_iter()
    .any(|header| match std::str::from_utf8(content_type) {
        Ok(content_type) => content_type.starts_with(header),
        Err(_) => false,
    }) {
        std::str::from_utf8(bytes).unwrap_or("[non-utf8 body]").to_string()
    } else {
        "[base64] ".to_string() + &BASE64_URL_SAFE.encode(bytes)
    }
}

async fn log_request(
    body: Body,
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap<HeaderValue>,
    version: &Version,
) -> Result<Bytes, (StatusCode, String)> {
    let bytes = body_to_bytes(body).await?;
    let body = body_to_string(&bytes, headers);

    tracing::debug!(
        "request:\n{} {} {:?}\n{}\n\n{}",
        method,
        uri,
        version,
        print_headers(headers),
        body
    );
    Ok(bytes)
}

async fn log_response(
    body: Body,
    status: StatusCode,
    headers: &HeaderMap<HeaderValue>,
    version: &Version,
) -> Result<Bytes, (StatusCode, String)> {
    let bytes = body_to_bytes(body).await?;
    let body = body_to_string(&bytes, headers);

    tracing::debug!(
        "response:\n{:?} {}\n{}\n\n{}",
        version,
        status,
        print_headers(headers),
        body
    );
    Ok(bytes)
}
