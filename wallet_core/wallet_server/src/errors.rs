use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::warn;

use openid4vc::ErrorStatusCode;

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub(crate) enum ErrorResponse<T> {
    #[cfg(feature = "issuance")]
    Basic(openid4vc::ErrorResponse<T>),
    #[cfg(feature = "disclosure")]
    Redirect(openid4vc::RedirectErrorResponse<T>),
}

impl<T> ErrorResponse<T> {
    #[cfg(feature = "issuance")]
    pub(crate) fn new_basic(error: impl Into<openid4vc::ErrorResponse<T>>) -> Self {
        Self::Basic(error.into())
    }

    #[cfg(feature = "disclosure")]
    pub(crate) fn new_redirect(error: impl Into<openid4vc::RedirectErrorResponse<T>>) -> Self {
        Self::Redirect(error.into())
    }
}

impl<T: ErrorStatusCode + Serialize + std::fmt::Debug> IntoResponse for ErrorResponse<T> {
    fn into_response(self) -> Response {
        warn!("Responding with error body: {:?}", &self);

        let status_code = match &self {
            #[cfg(feature = "issuance")]
            Self::Basic(response) => response,
            #[cfg(feature = "disclosure")]
            Self::Redirect(response) => &response.error_response,
        }
        .error
        .status_code();

        (status_code, Json(self)).into_response()
    }
}
