use core::result::Result as StdResult;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use nutype::nutype;
use tracing::warn;

#[nutype(derive(Debug, From, AsRef))]
pub struct Error(anyhow::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, self.as_ref().to_string()).into_response()
    }
}

pub type Result<T> = StdResult<T, Error>;
