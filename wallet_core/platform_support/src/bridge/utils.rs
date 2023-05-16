use std::fmt::Debug;

use super::get_bridge_collection;

pub use crate::utils::UtilitiesError;

// this is required to catch UnexpectedUniFFICallbackError
impl From<uniffi::UnexpectedUniFFICallbackError> for UtilitiesError {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::BridgingError { reason: value.reason }
    }
}

// the callback traits defined in the UDL, which we have write out here ourselves
pub trait UtilitiesBridge: Send + Sync + Debug {
    fn get_storage_path(&self) -> Result<String, UtilitiesError>;
}

pub fn get_utils_bridge() -> &'static dyn UtilitiesBridge {
    get_bridge_collection().utils.as_ref()
}
