use serde::{Deserialize, Serialize};
pub use wallet::PinError;

impl From<Result<(), PinError>> for PinResult {
    fn from(source: Result<(), PinError>) -> Self {
        match source {
            Ok(()) => PinResult::Ok,
            Err(err) => PinResult::Err(err),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PinResult {
    Ok,
    Err(PinError),
}
