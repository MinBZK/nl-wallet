use serde::{Deserialize, Serialize};
use wallet::pin::validation::PinValidationError;

impl From<Result<(), PinValidationError>> for PinValidationResult {
    fn from(source: Result<(), PinValidationError>) -> Self {
        match source {
            Ok(()) => PinValidationResult::Ok,
            Err(err) => match err {
                PinValidationError::NonDigits => PinValidationResult::NonDigitsError,
                PinValidationError::InvalidLength => PinValidationResult::InvalidLengthError,
                PinValidationError::TooFewUniqueDigits => PinValidationResult::TooFewUniqueDigitsError,
                PinValidationError::AscendingDigits => PinValidationResult::AscendingDigitsError,
                PinValidationError::DescendingDigits => PinValidationResult::DescendingDigitsError,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PinValidationResult {
    Ok,
    NonDigitsError,
    InvalidLengthError,
    TooFewUniqueDigitsError,
    AscendingDigitsError,
    DescendingDigitsError,
}
