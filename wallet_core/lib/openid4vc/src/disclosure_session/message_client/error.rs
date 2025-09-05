use error_category::ErrorCategory;
use http_utils::urls::BaseUrl;

use crate::errors::DisclosureErrorResponse;
use crate::errors::GetRequestErrorCode;
use crate::errors::PostAuthResponseErrorCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpMessageClientErrorType {
    Expired { can_retry: bool },
    Cancelled,
    Other,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum VpMessageClientError {
    #[error("HTTP request error: {0}")]
    #[category(expected)]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("auth request server error response: {0:?}")]
    AuthGetResponse(Box<DisclosureErrorResponse<GetRequestErrorCode>>),
    #[error("auth request server error response: {0:?}")]
    AuthPostResponse(Box<DisclosureErrorResponse<PostAuthResponseErrorCode>>),
    #[error("JWT error: {0}")]
    InvalidJwt(#[from] jwt::error::JwtError),
}

impl VpMessageClientError {
    pub fn error_type(&self) -> VpMessageClientErrorType {
        match self {
            // Consider the different error codes when getting the disclosure request.
            Self::AuthGetResponse(error) => match error.response_error() {
                GetRequestErrorCode::ExpiredEphemeralId => VpMessageClientErrorType::Expired { can_retry: true },
                GetRequestErrorCode::ExpiredSession => VpMessageClientErrorType::Expired { can_retry: false },
                GetRequestErrorCode::CancelledSession => VpMessageClientErrorType::Cancelled,
                _ => VpMessageClientErrorType::Other,
            },
            // Consider the different error codes when posting the disclosure response.
            Self::AuthPostResponse(error) => match error.response_error() {
                PostAuthResponseErrorCode::ExpiredSession => VpMessageClientErrorType::Expired { can_retry: false },
                PostAuthResponseErrorCode::CancelledSession => VpMessageClientErrorType::Cancelled,
                _ => VpMessageClientErrorType::Other,
            },
            // Any other reported error is classified under `VpMessageClientErrorType::Other`.
            _ => VpMessageClientErrorType::Other,
        }
    }

    pub fn redirect_uri(&self) -> Option<&BaseUrl> {
        match self {
            Self::AuthGetResponse(response) => response.redirect_uri.as_ref(),
            Self::AuthPostResponse(response) => response.redirect_uri.as_ref(),
            _ => None,
        }
    }
}

impl From<DisclosureErrorResponse<GetRequestErrorCode>> for VpMessageClientError {
    fn from(value: DisclosureErrorResponse<GetRequestErrorCode>) -> Self {
        Self::AuthGetResponse(value.into())
    }
}

impl From<DisclosureErrorResponse<PostAuthResponseErrorCode>> for VpMessageClientError {
    fn from(value: DisclosureErrorResponse<PostAuthResponseErrorCode>) -> Self {
        Self::AuthPostResponse(value.into())
    }
}
