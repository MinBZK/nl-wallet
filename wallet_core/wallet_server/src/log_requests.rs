use axum::{
    middleware::Next,
    response::{IntoResponse, Response},
};
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
        .map(|(key, val)| format!("{}: {}", key, val.to_str().unwrap()))
        .collect::<Vec<_>>()
        .join("\n")
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
    tracing::debug!(
        "request:\n{} {} {:?}\n{}\n\n{}",
        method,
        uri,
        version,
        print_headers(headers),
        std::str::from_utf8(&bytes).unwrap()
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
    tracing::debug!(
        "response:\n{:?} {}\n{}\n\n{}",
        version,
        status,
        print_headers(headers),
        std::str::from_utf8(&bytes).unwrap()
    );
    Ok(bytes)
}
