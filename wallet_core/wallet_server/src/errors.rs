use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::warn;

use openid4vc::ErrorStatusCode;

/// Newtype of [`openid4vc::ErrorResponse`] so that we can implement [`IntoResponse`] on it.
#[derive(Serialize, Debug)]
pub(crate) struct ErrorResponse<T>(pub(crate) openid4vc::ErrorResponse<T>);

impl<T: ErrorStatusCode + Serialize + std::fmt::Debug> IntoResponse for ErrorResponse<T> {
    fn into_response(self) -> Response {
        warn!("{:?}", &self);
        (self.0.error.status_code(), Json(self)).into_response()
    }
}
