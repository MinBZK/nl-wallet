use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::{header, HeaderValue, StatusCode};
use serde::Serialize;
use tracing::info;

use wallet_common::http_error::{ErrorData, APPLICATION_PROBLEM_JSON};

use crate::{gba, haal_centraal};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("GBA error: {0}")]
    Gba(#[from] gba::error::Error),
    #[error("Error converting GBA-V XML to Haal-Centraal JSON: {0}")]
    Conversion(#[from] haal_centraal::Error),
}

impl From<&Error> for StatusCode {
    fn from(value: &Error) -> Self {
        match value {
            Error::Gba(e) => e.into(),
            Error::Conversion(e) => e.into(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        info!("error handling request: {:?}", &self);

        let status_code: StatusCode = (&self).into();

        #[derive(Serialize)]
        struct InnerType {
            r#type: String,
        }

        let error_data = ErrorData {
            typ: match self {
                Error::Gba(_) => InnerType {
                    r#type: String::from("gba_error"),
                },
                Error::Conversion(_) => InnerType {
                    r#type: String::from("conversion_error"),
                },
            },
            title: self.to_string(),
        };

        (
            status_code,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(APPLICATION_PROBLEM_JSON.as_ref()),
            )],
            Json(error_data),
        )
            .into_response()
    }
}
