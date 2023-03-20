use serde::{Deserialize, Serialize};
use wallet::PinError;

impl From<Result<(), PinError>> for PinResult {
    fn from(source: Result<(), PinError>) -> Self {
        match source {
            Ok(()) => PinResult::Ok,
            Err(err) => match err {
                PinError::NonDigits => PinResult::OtherError,
                PinError::InvalidLength => PinResult::OtherError,
                PinError::TooFewUniqueDigits => PinResult::TooFewUniqueDigitsError,
                PinError::AscendingDigits => PinResult::SequentialDigitsError,
                PinError::DescendingDigits => PinResult::SequentialDigitsError,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PinResult {
    Ok,
    TooFewUniqueDigitsError,
    SequentialDigitsError,
    OtherError,
}
