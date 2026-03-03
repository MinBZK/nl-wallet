pub mod hardware;

#[cfg(feature = "mock_iso18013_5")]
pub mod mock;
#[cfg(any(all(feature = "mock_iso18013_5", test), feature = "hardware_integration_test"))]
pub mod test;

pub use crate::bridge::iso18013_5::Iso18013_5Error;

pub trait Iso18013_5SessionManager {
    async fn start_qr_handover() -> Result<String, Iso18013_5Error>;

    // async fn send_device_response(response: Vec<u8>) -> Result<(), Iso18013_5Error>;
    // async fn stop_ble_server() -> Result<(), Iso18013_5Error>;
}
