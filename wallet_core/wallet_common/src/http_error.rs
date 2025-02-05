use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use http::StatusCode;
use mime::Mime;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::DisplayFromStr;
use serde_with::TryFromInto;

pub static APPLICATION_PROBLEM_JSON: LazyLock<Mime> =
    LazyLock::new(|| "application/problem+json".parse().expect("could not parse MIME type"));

/// The HTTP body for an error response, as defined in RFC 7807.
/// If the `axum` feature is enabled, `IntoResponse` will be implemented for this
/// type, using `application/problem+json` as `Content-Type`.
///
/// See <https://datatracker.ietf.org/doc/html/rfc7807>
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpJsonErrorBody<T>
where
    T: Display + FromStr,
    T::Err: Display,
{
    /// A short string category for the error. This is the only mandatory field. Its type is generic and encoding to
    /// and from a string is enforced by the [`Display`] and [`FromStr`] trait bounds.
    #[serde_as(as = "DisplayFromStr")]
    pub r#type: T,

    /// A short summary of the error type.
    pub title: Option<String>,

    /// An HTTP status code for the error response. In the `IntoResponse` implementation this will be used as
    /// the actual status code. If this field is `None`, as 500 status code will be used.
    #[serde_as(as = "Option<TryFromInto<u16>>")]
    pub status: Option<StatusCode>,

    /// Detailed and specific information about the error.
    pub detail: Option<String>,

    /// A unique URI referencing the error.
    pub instance: Option<String>,

    /// This may contain extra key-value pairs of the JSON object, as allowed by the RFC. As this uses both the [`Map`]
    /// and [`Value`] types from the `serde_json` crate, (de)serialization of [`HttpJsonErrorBody`] instances is
    /// restricted to JSON.
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
#[error("({type}): {detail}")]
pub struct HttpJsonError<T> {
    r#type: T,
    detail: String,
    data: Map<String, Value>,
}

/// This trait is to be used as a bound for the `r#type` field of [`HttpJsonError`].
/// The result of this is that, when using the [`HttpJsonError`] type as a bridge
/// for conversion to [`HttpJsonErrorBody`], every distinct error type always
/// resolves to the same summary and HTTP status code.
pub trait HttpJsonErrorType {
    fn title(&self) -> String;
    fn status_code(&self) -> StatusCode;
}

impl<T> HttpJsonError<T> {
    pub fn new(r#type: T, detail: String, data: Map<String, Value>) -> Self {
        HttpJsonError { r#type, detail, data }
    }

    /// This convenience constructor allows for a [`HttpJsonError`] to be built
    /// from any type that both implements [`Error`] and can be converted into
    /// its associated error type, without extra data.
    pub fn from_error(error: impl Error + Into<T>) -> Self {
        let detail = error.to_string();

        Self::new(error.into(), detail, Default::default())
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
    use std::fmt::Display;
    use std::str::FromStr;

    use axum::response::IntoResponse;
    use axum::response::Response;
    use axum::Json;
    use http::header::CONTENT_TYPE;
    use http::HeaderValue;
    use http::StatusCode;

    use super::HttpJsonError;
    use super::HttpJsonErrorBody;
    use super::HttpJsonErrorType;
    use super::APPLICATION_PROBLEM_JSON;

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
    use http::StatusCode;
    use serde_json::json;
    use serde_json::Value;

    use super::HttpJsonErrorBody;
    use super::HttpJsonErrorType;

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
    mod axum {
        use axum::body;
        use axum::response::IntoResponse;
        use http::StatusCode;
        use serde_json::json;
        use serde_json::Value;

        use super::super::HttpJsonError;
        use super::super::HttpJsonErrorBody;
        use super::TestErrorType;

        #[tokio::test]
        async fn test_http_json_error_body_into_response() {
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

            let body = body::to_bytes(response.into_body(), 1024).await.unwrap();
            let json = serde_json::from_slice::<Value>(&body).unwrap();
            let expected_json = json!({
                "type": "foobar",
                "title": "A foobar error.",
                "status": 402,
                "detail": "Something happened.",
            });

            assert_eq!(json, expected_json);

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

        #[tokio::test]
        async fn test_http_json_error_into_response() {
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

            let body = body::to_bytes(response.into_body(), 1024).await.unwrap();
            let json = serde_json::from_slice::<Value>(&body).unwrap();
            let expected_json = json!({
                "type": "loop",
                "title": "Loop detected.",
                "status": 508,
                "detail": "Caught in a time vortex.",
            });

            assert_eq!(json, expected_json);
        }
    }
}
