use axum::{
    headers::HeaderValue,
    response::{IntoResponse, Response},
};
use reqwest::{header, StatusCode};
use serde::Serialize;

use nl_wallet_mdoc::utils::serialization::cbor_serialize;

// CBOR Response
// based on https://github.com/tokio-rs/axum/blob/main/axum/src/json.rs
pub struct Cbor<T>(pub T);

impl<T> From<T> for Cbor<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for Cbor<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match cbor_serialize(&self.0) {
            Ok(buf) => (
                [(header::CONTENT_TYPE, HeaderValue::from_static("application/cbor"))], // https://datatracker.ietf.org/doc/html/rfc7049#section-7.3
                buf,
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}
