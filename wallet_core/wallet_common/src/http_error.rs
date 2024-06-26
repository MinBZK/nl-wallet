use std::{error::Error, fmt::Display, str::FromStr};

use http::StatusCode;
use mime::Mime;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr, TryFromInto};

pub static APPLICATION_PROBLEM_JSON: Lazy<Mime> =
    Lazy::new(|| "application/problem+json".parse().expect("could not parse MIME type"));

/// This type represents the HTTP body for an error response, as defined in RFC 7807.
/// If the `axum` feature is enabled, `IntoResponse` will be implemented for this
/// type, using `application/problem+json` as `Content-Type`.
///
/// * `r#type` - A short string category for the error. This is the only mandatory
///              field. Its type is generic and encoding to and from a string is
///              enforced by the [`Display`] and [`FromStr`] trait bounds.
/// * `title` - An optional a short summary of the error type.
/// * `status` - An optional HTTP status code for the error response. In the
///              `IntoResponse` implementation this will be used as the actual status
///              code. If this field is `None`, as 500 status code will be used.
/// * `detail` - Optional detailed and specific information about the error.
/// * `instance` - An optional unique URI referencing the error.
/// * `extra` - This may contain extra key-value pairs of the JSON object, as allowed
///             by the RFC. As this uses both the [`Map`] and [`Value`] types from
///             the `serde_json` crate, (de)serialization of [`HttpJsonErrorBody`]
///             instances is restricted to JSON.
///
/// See: https://datatracker.ietf.org/doc/html/rfc7807
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpJsonErrorBody<T>
where
    T: Display + FromStr,
    T::Err: Display,
{
    #[serde_as(as = "DisplayFromStr")]
    pub r#type: T,
    pub title: Option<String>,
    #[serde_as(as = "Option<TryFromInto<u16>>")]
    pub status: Option<StatusCode>,
    pub detail: Option<String>,
    pub instance: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// This convenience type can be used as a bridge from types that implement
/// [`Error`] to [`HttpJsonErrorBody`]. It contains a type (which prescribes
/// the error type string, title and status code), the details of the
/// error and any extra key-value pairs of data pertaining to the error.
///
/// If the `axum` feature is enabled, `IntoResponse` will also be implemented
/// for this type.
#[derive(Debug, thiserror::Error)]
#[error("({r#type}): {detail}")]
pub struct HttpJsonError<T> {
    r#type: T,
    detail: String,
    data: Map<String, Value>,
}

/// This trait is to be used as a bound for the `r#type` field of `HttpJsonError`.
/// The intention of it is that every distinct error type always resolves to the
/// same summary and HTTP status code.
pub trait HttpJsonErrorType {
    fn title(&self) -> String;
    fn status_code(&self) -> StatusCode;
}

impl<T> HttpJsonError<T> {
    pub fn new(r#type: T, description: String, data: Map<String, Value>) -> Self {
        HttpJsonError {
            r#type,
            detail: description,
            data,
        }
    }

    /// This convenience constructor allows for a [`HttpJsonError`] to be built
    /// from any type that both implements [`Error`] and can be converted into
    /// its associated error type, without extra data.
    pub fn from_error(error: impl Error + Into<T>) -> Self {
        let description = error.to_string();

        Self::new(error.into(), description, Default::default())
    }
}

impl<T> From<HttpJsonError<T>> for HttpJsonErrorBody<T>
where
    T: HttpJsonErrorType + Display + FromStr,
    T::Err: Display,
{
    fn from(value: HttpJsonError<T>) -> Self {
        let title = Some(value.r#type.title());
        let status = Some(value.r#type.status_code());

        HttpJsonErrorBody {
            r#type: value.r#type,
            title,
            status,
            detail: Some(value.detail),
            instance: None,
            extra: value.data,
        }
    }
}

#[cfg(feature = "axum")]
mod axum {
    use ::axum::{
        response::{IntoResponse, Response},
        Json,
    };
    use http::{header::CONTENT_TYPE, HeaderValue};

    use super::*;

    impl<T> IntoResponse for HttpJsonErrorBody<T>
    where
        T: Display + FromStr,
        T::Err: Display,
    {
        fn into_response(self) -> Response {
            (
                // Use a sensible default status code when none is provided.
                self.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                [(
                    CONTENT_TYPE,
                    HeaderValue::from_static(APPLICATION_PROBLEM_JSON.as_ref()),
                )],
                Json(self),
            )
                .into_response()
        }
    }

    impl<T> IntoResponse for HttpJsonError<T>
    where
        T: HttpJsonErrorType + Display + FromStr,
        T::Err: Display,
    {
        fn into_response(self) -> Response {
            HttpJsonErrorBody::<T>::from(self).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
    #[strum(serialize_all = "snake_case")]
    enum TestErrorType {
        Teapot,
        Loop,
    }

    impl HttpJsonErrorType for TestErrorType {
        fn title(&self) -> String {
            match self {
                Self::Teapot => "I'm a teapot".to_string(),
                Self::Loop => "Loop detected.".to_string(),
            }
        }

        fn status_code(&self) -> StatusCode {
            match self {
                Self::Teapot => StatusCode::IM_A_TEAPOT,
                Self::Loop => StatusCode::LOOP_DETECTED,
            }
        }
    }

    #[test]
    fn test_http_json_error_body_serialization_basic() {
        let error_body = HttpJsonErrorBody {
            r#type: "error_type".to_string(),
            title: None,
            status: None,
            detail: None,
            instance: None,
            extra: Default::default(),
        };

        let json = serde_json::to_string(&error_body)
            .and_then(|body| serde_json::from_str::<Value>(&body))
            .expect("should serialize and deserialize HttpJsonErrorBody to JSON");
        let expected_json = json!({"type": "error_type"});

        assert_eq!(json, expected_json);

        let parsed_error_body =
            serde_json::from_value::<HttpJsonErrorBody<String>>(json).expect("should deserialize HttpJsonErrorBody");

        assert_eq!(parsed_error_body.r#type, "error_type");
    }

    #[test]
    fn test_http_json_error_body_serialization_full() {
        let error_body = HttpJsonErrorBody {
            r#type: "some_type".to_string(),
            title: Some("The summary of the error type.".to_string()),
            status: Some(StatusCode::BAD_REQUEST),
            detail: Some("SomeError: ThisHappened".to_string()),
            instance: Some("https://example.com".to_string()),
            extra: [
                ("string".to_string(), "value".to_string().into()),
                ("number".to_string(), 1234.into()),
            ]
            .into_iter()
            .collect(),
        };

        let json = serde_json::to_string(&error_body)
            .and_then(|body| serde_json::from_str::<Value>(&body))
            .expect("should serialize and deserialize HttpJsonErrorBody to JSON");
        let expected_json = json!({
            "type": "some_type",
            "title": "The summary of the error type.",
            "status": 400,
            "detail": "SomeError: ThisHappened",
            "instance": "https://example.com",
            "string": "value",
            "number": 1234
        });

        assert_eq!(json, expected_json);

        let parsed_error_body =
            serde_json::from_value::<HttpJsonErrorBody<String>>(json).expect("should deserialize HttpJsonErrorBody");

        assert_eq!(parsed_error_body.r#type, "some_type");
    }

    #[cfg(feature = "axum")]
    #[tokio::test]
    async fn test_http_json_error_body_into_response() {
        use ::axum::response::IntoResponse;

        let error_body = HttpJsonErrorBody {
            r#type: "foobar".to_string(),
            title: Some("A foobar error.".to_string()),
            status: Some(StatusCode::PAYMENT_REQUIRED),
            detail: Some("Something happened.".to_string()),
            instance: None,
            extra: Default::default(),
        };
        let response = error_body.into_response();

        assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/problem+json"
        );

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json = serde_json::from_slice::<Value>(&body).unwrap();

        assert_eq!(json.get("type").unwrap(), "foobar");
        assert_eq!(json.get("title").unwrap(), "A foobar error.");
        assert_eq!(json.get("status").unwrap(), 402);
        assert_eq!(json.get("detail").unwrap(), "Something happened.");

        let error_body = HttpJsonErrorBody {
            r#type: "simple".to_string(),
            title: None,
            status: None,
            detail: None,
            instance: None,
            extra: Default::default(),
        };
        let response = error_body.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[cfg(feature = "axum")]
    #[tokio::test]
    async fn test_http_json_error_into_response() {
        use ::axum::response::IntoResponse;

        let error = HttpJsonError::new(
            TestErrorType::Loop,
            "Caught in a time vortex.".to_string(),
            Default::default(),
        );
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::LOOP_DETECTED);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/problem+json"
        );

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json = serde_json::from_slice::<Value>(&body).unwrap();

        assert_eq!(json.get("type").unwrap(), "loop");
        assert_eq!(json.get("title").unwrap(), "Loop detected.");
        assert_eq!(json.get("status").unwrap(), 508);
        assert_eq!(json.get("detail").unwrap(), "Caught in a time vortex.");
    }
}
