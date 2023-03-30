use once_cell::sync::OnceCell;
use std::{fmt::Debug, sync::Mutex};

use crate::utils::error::UtilitiesError;

// this is required to catch UnexpectedUniFFICallbackError
impl From<uniffi::UnexpectedUniFFICallbackError> for UtilitiesError {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::BridgingError {
            reason: value.reason,
        }
    }
}

// the callback traits defined in the UDL, which we have write out here ourselves
pub trait UtilitiesBridge: Send + Sync + Debug {
    fn get_storage_path(&self) -> Result<String, UtilitiesError>;
}

// protect utilities with mutex
pub static UTILITIES: OnceCell<Mutex<Box<dyn UtilitiesBridge>>> = OnceCell::new();

pub fn init_utilities(bridge: Box<dyn UtilitiesBridge>) {
    let utilities = Mutex::new(bridge);
    // crash if STORAGE was already set
    assert!(
        UTILITIES.set(utilities).is_ok(),
        "Cannot call init_utilities() more than once"
    )
}
