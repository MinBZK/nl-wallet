use std::fmt::Debug;

use async_trait::async_trait;

use super::get_platform_support;

/// Implementation of `Iso18013_5Error` from the UDL file.
#[derive(Debug, thiserror::Error)]
pub enum Iso18013_5Error {
    #[error("platform error: {reason}")]
    PlatformError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

// the callback traits defined in the UDL, which we have write out here ourselves
#[uniffi::trait_interface]
#[async_trait]
pub trait Iso18013_5Bridge: Send + Sync + Debug {
    async fn start_qr_handover(&self) -> Result<String, Iso18013_5Error>;

    // async fn send_device_response(&self, response: Vec<u8>) -> Result<(), Iso18013_5Error>;
    // async fn stop_ble_server(&self) -> Result<(), Iso18013_5Error>;
}

/// Convenience function to access the a reference to `Iso18013_5Bridge`,
/// as set by by the native implementation.
pub fn get_iso18013_5_bridge() -> &'static dyn Iso18013_5Bridge {
    get_platform_support().iso18013_5.as_ref()
}
