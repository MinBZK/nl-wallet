use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

use derive_more::Constructor;
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;

pub trait ErrorWithCode: Error {
    type ErrorCode: Display;

    fn error_code(&self) -> Self::ErrorCode;
}

/// A type that wraps a `Box<dyn>` error and implements both the `Error` and `ToErrorCode` traits. This allows it to be
/// used as an error source for `thiserror` error types.
#[derive(Debug, derive_more::Display)]
pub struct BoxedErrorWithCode<T>(Box<dyn ErrorWithCode<ErrorCode = T> + Send + Sync + 'static>);

impl<T> BoxedErrorWithCode<T> {
    pub fn new(error: impl ErrorWithCode<ErrorCode = T> + Send + Sync + 'static) -> Self {
        Self(Box::new(error))
    }
}

// Implement the `Error` trait manually, in order to  work around a limitation of `thiserror`'s `#[source]` annotation.
// Unfortunately this annotation does not appear to work with boxed errors that are not `Box<dyn Error>`, but instead a
// superset of traits.
impl<T> Error for BoxedErrorWithCode<T>
where
    T: Debug,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self(inner) = self;

        Some(inner.as_ref())
    }
}

impl<T> ErrorWithCode for BoxedErrorWithCode<T>
where
    T: Debug + Display,
{
    type ErrorCode = T;

    fn error_code(&self) -> Self::ErrorCode {
        let Self(inner) = self;

        inner.error_code()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
pub enum RemoteErrorCode<T> {
    Known(T),
    Unknown(String),
}

impl<T> FromStr for RemoteErrorCode<T>
where
    T: FromStr,
{
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error_type = match s.parse() {
            Ok(error_type) => Self::Known(error_type),
            Err(_) => Self::Unknown(s.to_string()),
        };

        Ok(error_type)
    }
}

pub type RemoteErrorResponse<T> = ErrorResponse<RemoteErrorCode<T>>;

/// Describes an error that occurred when processing an HTTP endpoint from the OAuth/OpenID protocol family.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse<T> {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(bound(serialize = "T: Display", deserialize = "T: FromStr, T::Err: Display"))]
    pub error: T,
    pub error_description: Option<String>,
    pub error_uri: Option<Box<Url>>,
}

#[cfg(any(test, feature = "test"))]
impl<T> From<ErrorResponse<T>> for ErrorResponse<RemoteErrorCode<T>> {
    fn from(value: ErrorResponse<T>) -> Self {
        let ErrorResponse {
            error,
            error_description,
            error_uri,
        } = value;

        Self {
            error: RemoteErrorCode::Known(error),
            error_description,
            error_uri,
        }
    }
}

impl<E, T> From<E> for ErrorResponse<T>
where
    E: Display + ErrorWithCode<ErrorCode = T>,
{
    fn from(value: E) -> Self {
        Self {
            error: value.error_code(),
            error_description: Some(value.to_string()),
            error_uri: None,
        }
    }
}

pub type RemoteAuthorizationErrorResponse<T> = AuthorizationErrorResponse<RemoteErrorCode<T>>;

/// Wrapper of [`ErrorResponse`] that adds the optional `state` parameter used by authorization error responses.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationErrorResponse<T> {
    #[serde(
        flatten,
        bound(serialize = "T: Display", deserialize = "T: FromStr, T::Err: Display")
    )]
    pub error_response: ErrorResponse<T>,
    pub state: Option<String>,
}

impl<T> AuthorizationErrorResponse<T> {
    pub fn error(&self) -> &T {
        &self.error_response.error
    }
}

#[cfg(any(test, feature = "test"))]
impl<T> From<AuthorizationErrorResponse<T>> for AuthorizationErrorResponse<RemoteErrorCode<T>> {
    fn from(value: AuthorizationErrorResponse<T>) -> Self {
        let AuthorizationErrorResponse { error_response, state } = value;

        Self {
            error_response: error_response.into(),
            state,
        }
    }
}

/// Describes an error that occured at a HTTP(S) endpoint that is meant to be returned in a 303 See Other redirect.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectErrorResponse<T> {
    pub auth_error_response: AuthorizationErrorResponse<T>,
    pub redirect_uri: Box<Url>,
}

/// Wraps an [`Error`] type that can be returned in a HTTP(S) redirect.
#[derive(Debug, thiserror::Error, Constructor)]
#[error("{error}")]
pub struct RedirectError<E>
where
    E: Error,
{
    #[source]
    pub error: E,
    pub redirect_uri: Box<Url>,
    pub state: Option<String>,
}

impl<E, T> From<RedirectError<E>> for RedirectErrorResponse<T>
where
    E: Error + ErrorWithCode<ErrorCode = T>,
{
    fn from(value: RedirectError<E>) -> Self {
        let RedirectError {
            error,
            redirect_uri,
            state,
        } = value;

        Self {
            auth_error_response: AuthorizationErrorResponse {
                error_response: ErrorResponse::from(error),
                state,
            },
            redirect_uri,
        }
    }
}

/// Describes an error that occured at a HTTP(S) endpoint that is meant to be returned either as a status code and
/// plain-text body or in a 303 See Other redirect.
#[derive(Debug, Clone)]
pub enum BodyOrRedirectErrorResponse<T> {
    Body { status_code: StatusCode, body_text: String },
    Redirect(RedirectErrorResponse<T>),
}

impl<T> BodyOrRedirectErrorResponse<T> {
    pub fn new_body(status_code: StatusCode, body_text: String) -> Self {
        Self::Body { status_code, body_text }
    }

    pub fn new_redirect(redirect_response: RedirectErrorResponse<T>) -> Self {
        Self::Redirect(redirect_response)
    }
}

impl<E, T> From<RedirectError<E>> for BodyOrRedirectErrorResponse<T>
where
    E: Error + ErrorWithCode<ErrorCode = T>,
{
    fn from(value: RedirectError<E>) -> Self {
        Self::new_redirect(value.into())
    }
}

pub trait ErrorStatusCode {
    fn status_code(&self) -> StatusCode;
}

/// The list of error codes that can result from an Authorization Request.
///
/// See: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1>
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AuthorizationErrorCode {
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
}

/// The list of error codes that can result from the Token Request.
///
/// See <https://www.rfc-editor.org/rfc/rfc6749.html#section-5.2>.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TokenErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,

    /// This can be returned in case of internal server errors, i.e. with HTTP status code 5xx.
    /// This error type is not defined in the specs, but then again the entire HTTP response in case
    /// 5xx status codes is not defined by the specs, so we have freedom to return what we want.
    ServerError,
}

impl ErrorStatusCode for TokenErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest
            | Self::InvalidGrant
            | Self::UnauthorizedClient
            | Self::UnsupportedGrantType
            | Self::InvalidScope => StatusCode::BAD_REQUEST,
            Self::InvalidClient => StatusCode::UNAUTHORIZED,
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "axum")]
mod axum {
    use std::fmt::Debug;
    use std::fmt::Display;

    use axum::Json;
    use axum::response::IntoResponse;
    use axum::response::Redirect;
    use axum::response::Response;
    use tracing::warn;

    use super::BodyOrRedirectErrorResponse;
    use super::ErrorResponse;
    use super::ErrorStatusCode;
    use super::RedirectErrorResponse;

    impl<T> IntoResponse for ErrorResponse<T>
    where
        T: ErrorStatusCode + Display + Debug,
    {
        fn into_response(self) -> Response {
            warn!("Responding with error body: {:?}", &self);

            (self.error.status_code(), Json(self)).into_response()
        }
    }

    impl<T> IntoResponse for RedirectErrorResponse<T>
    where
        T: Display + Debug,
    {
        fn into_response(self) -> Response {
            warn!("Responding with error redirect: {:?}", &self);

            let mut redirect_uri = self.redirect_uri;

            {
                let mut query_pairs = redirect_uri.query_pairs_mut();

                query_pairs.append_pair("error", &self.auth_error_response.error_response.error.to_string());

                if let Some(error_description) = self.auth_error_response.error_response.error_description.as_deref() {
                    query_pairs.append_pair("error_description", error_description);
                }

                if let Some(error_uri) = self.auth_error_response.error_response.error_uri.as_ref() {
                    query_pairs.append_pair("error_uri", error_uri.as_str());
                }

                if let Some(state) = self.auth_error_response.state.as_deref() {
                    query_pairs.append_pair("state", state);
                }
            }

            Redirect::to(redirect_uri.as_str()).into_response()
        }
    }

    impl<T> IntoResponse for BodyOrRedirectErrorResponse<T>
    where
        T: Display + Debug,
    {
        fn into_response(self) -> Response {
            match self {
                Self::Body { status_code, body_text } => {
                    warn!("Responding with error body ({status_code}): {body_text}");

                    (status_code, body_text).into_response()
                }
                Self::Redirect(redirect_response) => redirect_response.into_response(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use axum::response::IntoResponse;
        use derive_more::Display;
        use http::header;
        use url::Url;

        use super::super::ErrorWithCode;
        use super::super::RedirectError;
        use super::super::RedirectErrorResponse;

        #[test]
        fn test_redirect_error_into_response() {
            #[derive(Debug, thiserror::Error)]
            #[error("{0}")]
            struct ExampleError(String);

            #[derive(Debug, Display)]
            #[display("something_happened")]
            struct ExampleErrorCode;

            impl ErrorWithCode for ExampleError {
                type ErrorCode = ExampleErrorCode;

                fn error_code(&self) -> Self::ErrorCode {
                    ExampleErrorCode
                }
            }

            let example_error = ExampleError("Something happened 猫".to_string());
            let redirect_uri = "http://example.com/redirect?foo=bar".parse().unwrap();
            let state = "wallet_state".to_string();

            let redirect_error = RedirectError::new(example_error, Box::new(redirect_uri), Some(state));
            let mut error_response = RedirectErrorResponse::<ExampleErrorCode>::from(redirect_error);
            error_response.auth_error_response.error_response.error_uri =
                Some(Box::new("https://example.com/error-info".parse().unwrap()));

            let response = error_response.into_response();

            let location_header = response
                .headers()
                .get(header::LOCATION)
                .expect("response should have Location header");

            let url = location_header
                .to_str()
                .unwrap()
                .parse::<Url>()
                .expect("Location header should contain a URL");

            assert_eq!(
                url.query(),
                Some(
                    "foo=bar&error=something_happened&error_description=Something+happened+%E7%8C%AB&error_uri=https%\
                     3A%2F%2Fexample.com%2Ferror-info&state=wallet_state"
                )
            );
        }
    }
}
