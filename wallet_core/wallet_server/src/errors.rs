use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_with::skip_serializing_none;
use tracing::warn;

use openid4vc::ErrorStatusCode;
use wallet_common::config::wallet_config::BaseUrl;

/// Wrapper of [`openid4vc::ErrorResponse`] that implements [`IntoResponse`] and has an optional redirect URI.
#[skip_serializing_none]
#[derive(Serialize, Debug)]
pub(crate) struct ErrorResponse<T> {
    #[serde(flatten)]
    pub(crate) error_response: openid4vc::ErrorResponse<T>,
    pub(crate) redirect_uri: Option<BaseUrl>,
}

impl<T: ErrorStatusCode + Serialize + std::fmt::Debug> IntoResponse for ErrorResponse<T> {
    fn into_response(self) -> Response {
        warn!("{:?}", &self);
        (self.error_response.error.status_code(), Json(self)).into_response()
    }
}

impl<T> ErrorResponse<T> {
    pub(crate) fn new(err: impl Into<openid4vc::ErrorResponse<T>>) -> Self {
        Self {
            error_response: err.into(),
            redirect_uri: None,
        }
    }

    #[cfg(feature = "disclosure")]
    pub(crate) fn with_uri(
        err: openid4vc::verifier::WithRedirectUri<impl Into<openid4vc::ErrorResponse<T>> + std::fmt::Debug>,
    ) -> Self {
        Self {
            error_response: err.error.into(),
            redirect_uri: err.redirect_uri,
        }
    }
}
