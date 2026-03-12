use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;

use super::get_platform_support;

/// Implementation of `CloseProximityDisclosureError` from the UDL file.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CloseProximityDisclosureError {
    #[error("platform error: {reason}")]
    PlatformError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

// the callback traits defined in the UDL, which we have to write out here ourselves
#[uniffi::trait_interface]
#[async_trait]
pub trait CloseProximityDisclosureBridge: Send + Sync + Debug {
    async fn start_qr_handover(
        &self,
        channel: Arc<dyn CloseProximityDisclosureChannel>,
    ) -> Result<String, CloseProximityDisclosureError>;

    async fn send_device_response(&self, response: Vec<u8>) -> Result<(), CloseProximityDisclosureError>;

    async fn stop_ble_server(&self) -> Result<(), CloseProximityDisclosureError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseProximityDisclosureUpdate {
    Connecting,
    Connected,
    SessionEstablished {
        session_transcript: Vec<u8>,
        device_request: Vec<u8>,
    },
    Closed,
    Error {
        error: CloseProximityDisclosureError,
    },
}

#[uniffi::trait_interface]
#[async_trait]
pub trait CloseProximityDisclosureChannel: Send + Sync {
    async fn send_update(&self, update: CloseProximityDisclosureUpdate) -> Result<(), CloseProximityDisclosureError>;
}

/// Convenience function to access the a reference to `CloseProximityDisclosureBridge`,
/// as set by by the native implementation.
pub fn get_close_proximity_disclosure_bridge() -> &'static dyn CloseProximityDisclosureBridge {
    get_platform_support().close_proximity_disclosure.as_ref()
}
