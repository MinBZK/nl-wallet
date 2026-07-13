use error_category::ErrorCategory;
use url::Url;

use crate::errors::GetAuthRequestErrorCode;
use crate::errors::PostAuthResponseErrorCode;
use crate::errors::RemoteDisclosureErrorResponse;
use crate::errors::RemoteErrorCode;

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
    AuthGetResponse(Box<RemoteDisclosureErrorResponse<GetAuthRequestErrorCode>>),

    #[error("auth request server error response: {0:?}")]
    AuthPostResponse(Box<RemoteDisclosureErrorResponse<PostAuthResponseErrorCode>>),

    #[error("JWT error: {0}")]
    InvalidJwt(#[from] jwt::error::JwtParseError),
}

impl VpMessageClientError {
    pub fn error_type(&self) -> VpMessageClientErrorType {
        match self {
            // Consider the different error codes when getting the disclosure request.
            Self::AuthGetResponse(response) => match response.error() {
                RemoteErrorCode::Known(GetAuthRequestErrorCode::ExpiredEphemeralId) => {
                    VpMessageClientErrorType::Expired { can_retry: true }
                }
                RemoteErrorCode::Known(GetAuthRequestErrorCode::ExpiredSession) => {
                    VpMessageClientErrorType::Expired { can_retry: false }
                }
                RemoteErrorCode::Known(GetAuthRequestErrorCode::CancelledSession) => {
                    VpMessageClientErrorType::Cancelled
                }
                _ => VpMessageClientErrorType::Other,
            },
            // Consider the different error codes when posting the disclosure response.
            Self::AuthPostResponse(response) => match response.error() {
                RemoteErrorCode::Known(PostAuthResponseErrorCode::ExpiredSession) => {
                    VpMessageClientErrorType::Expired { can_retry: false }
                }
                RemoteErrorCode::Known(PostAuthResponseErrorCode::CancelledSession) => {
                    VpMessageClientErrorType::Cancelled
                }
                _ => VpMessageClientErrorType::Other,
            },
            // Any other reported error is classified under `VpMessageClientErrorType::Other`.
            _ => VpMessageClientErrorType::Other,
        }
    }

    pub fn redirect_uri(&self) -> Option<&Url> {
        match self {
            Self::AuthGetResponse(response) => response.redirect_uri.as_ref(),
            Self::AuthPostResponse(response) => response.redirect_uri.as_ref(),
            _ => None,
        }
    }
}

impl From<RemoteDisclosureErrorResponse<GetAuthRequestErrorCode>> for VpMessageClientError {
    fn from(value: RemoteDisclosureErrorResponse<GetAuthRequestErrorCode>) -> Self {
        Self::AuthGetResponse(value.into())
    }
}

impl From<RemoteDisclosureErrorResponse<PostAuthResponseErrorCode>> for VpMessageClientError {
    fn from(value: RemoteDisclosureErrorResponse<PostAuthResponseErrorCode>) -> Self {
        Self::AuthPostResponse(value.into())
    }
}
