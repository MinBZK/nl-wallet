use std::fmt::Debug;

use error_category::ErrorCategory;
use uniffi::UnexpectedUniFFICallbackError;

use super::get_platform_support;

// implementation of UtilitiesError from UDL
#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)] // reason field might leak sensitive data
pub enum UtilitiesError {
    #[error("platform error: {reason}")]
    PlatformError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

// this is required to catch UnexpectedUniFFICallbackError
impl From<UnexpectedUniFFICallbackError> for UtilitiesError {
    fn from(value: UnexpectedUniFFICallbackError) -> Self {
        Self::BridgingError { reason: value.reason }
    }
}

// the callback traits defined in the UDL, which we have write out here ourselves
pub trait UtilitiesBridge: Send + Sync + Debug {
    fn get_storage_path(&self) -> Result<String, UtilitiesError>;
}

pub fn get_utils_bridge() -> &'static dyn UtilitiesBridge {
    get_platform_support().utils.as_ref()
}
