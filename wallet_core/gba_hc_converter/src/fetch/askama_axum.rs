use askama::Template;
use axum::{
    http::{self, StatusCode},
    response::{IntoResponse, Response},
};
use axum_csrf::CsrfToken;

pub fn into_response<T: Template>(t: &T) -> Response {
    into_response_optional_csrf(None, t)
}

pub fn into_response_with_csrf<T: Template>(csrf_token: CsrfToken, t: &T) -> Response {
    into_response_optional_csrf(Some(csrf_token), t)
}

// workaround for: https://github.com/djc/askama/issues/810#issuecomment-1494522435
// inspired by: https://github.com/djc/askama/blob/main/askama_axum/src/lib.rs
fn into_response_optional_csrf<T: Template>(csrf_token: Option<CsrfToken>, t: &T) -> Response {
    match t.render() {
        Ok(body) => {
            let headers = [(http::header::CONTENT_TYPE, http::HeaderValue::from_static(T::MIME_TYPE))];
            (headers, csrf_token, body).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
