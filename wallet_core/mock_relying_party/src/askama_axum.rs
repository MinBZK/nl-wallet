use askama::Template;
use axum::http::StatusCode;
use axum::http::{self};
use axum::response::IntoResponse;
use axum::response::Response;

// workaround for: https://github.com/djc/askama/issues/810#issuecomment-1494522435
// from: https://github.com/djc/askama/blob/main/askama_axum/src/lib.rs
pub fn into_response<T: Template>(t: &T) -> Response {
    match t.render() {
        Ok(body) => {
            let headers = [(http::header::CONTENT_TYPE, http::HeaderValue::from_static(T::MIME_TYPE))];

            (headers, body).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
