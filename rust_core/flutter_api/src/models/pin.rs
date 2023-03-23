use serde::{Deserialize, Serialize};
use wallet::pin::PinValidationError;

impl From<Result<(), PinValidationError>> for PinValidationResult {
    fn from(source: Result<(), PinValidationError>) -> Self {
        match source {
            Ok(()) => PinValidationResult::Ok,
            Err(err) => match err {
                PinValidationError::NonDigits => PinValidationResult::OtherError,
                PinValidationError::InvalidLength => PinValidationResult::OtherError,
                PinValidationError::TooFewUniqueDigits => PinValidationResult::TooFewUniqueDigitsError,
                PinValidationError::AscendingDigits => PinValidationResult::SequentialDigitsError,
                PinValidationError::DescendingDigits => PinValidationResult::SequentialDigitsError,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PinValidationResult {
    Ok,
    TooFewUniqueDigitsError,
    SequentialDigitsError,
    OtherError,
}
