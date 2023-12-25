use axum::{
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::prelude::*;
use http::{HeaderMap, HeaderValue, Method, Request, StatusCode, Uri, Version};
use hyper::{body::Bytes, Body};

pub(crate) async fn log_request_response(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = log_request(body, &parts.method, &parts.uri, &parts.headers, &parts.version).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = log_response(body, parts.status, &parts.headers, &parts.version).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn body_to_bytes<B>(body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    hyper::body::to_bytes(body)
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

    let content_type = headers.get("content-type").map(|header| header.as_bytes());
    match content_type {
        Some(content_type) => match content_type {
            b"application/json" => std::str::from_utf8(bytes).unwrap_or("[non-utf8 body]").to_string(),
            b"application/x-www-form-urlencoded" => std::str::from_utf8(bytes).unwrap_or("[non-utf8 body]").to_string(),
            _ => "[base64] ".to_string() + &BASE64_URL_SAFE.encode(bytes),
        },
        None => "[base64] ".to_string() + &BASE64_URL_SAFE.encode(bytes),
    }
}

async fn log_request<B>(
    body: B,
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap<HeaderValue>,
    version: &Version,
) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
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

async fn log_response<B>(
    body: B,
    status: StatusCode,
    headers: &HeaderMap<HeaderValue>,
    version: &Version,
) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
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
