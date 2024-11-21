use http::StatusCode;

use wallet_common::http_error::HttpJsonError;
use wallet_common::http_error::HttpJsonErrorType;

use crate::gba;
use crate::haal_centraal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorType {
    Transport,
    Gba,
    Conversion,
}

#[derive(Debug, thiserror::Error, strum::EnumDiscriminants)]
pub enum Error {
    #[error("GBA error: {0}")]
    Gba(#[from] gba::error::Error),
    #[error("Error converting GBA-V XML to Haal-Centraal JSON: {0}")]
    Conversion(#[from] haal_centraal::Error),
}

impl HttpJsonErrorType for ErrorType {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Transport => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Gba | Self::Conversion => StatusCode::PRECONDITION_FAILED,
        }
    }

    fn title(&self) -> String {
        match self {
            Self::Transport => "HTTP transport error".to_string(),
            Self::Gba => "GBA error".to_string(),
            Self::Conversion => "Conversion error".to_string(),
        }
    }
}

impl From<Error> for ErrorType {
    fn from(value: Error) -> Self {
        match value {
            Error::Gba(gba::error::Error::Transport(_)) => Self::Transport,
            Error::Gba(_) => Self::Gba,
            Error::Conversion(_) => Self::Conversion,
        }
    }
}

impl From<Error> for HttpJsonError<ErrorType> {
    fn from(value: Error) -> Self {
        HttpJsonError::from_error(value)
    }
}
