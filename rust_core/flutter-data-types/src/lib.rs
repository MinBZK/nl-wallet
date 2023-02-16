use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum PinError {
    NonDigits,
    InvalidLength,
    TooLittleUniqueDigits,
    AscendingDigits,
    DescendingDigits,
}

impl From<Result<(), PinError>> for PinResult {
    fn from(source: Result<(), PinError>) -> Self {
        match source {
            Ok(()) => PinResult::Ok,
            Err(err) => PinResult::Err(err),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum PinResult {
    Ok,
    Err(PinError),
}
