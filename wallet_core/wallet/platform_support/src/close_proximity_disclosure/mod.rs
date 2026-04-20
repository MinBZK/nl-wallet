pub mod hardware;

#[cfg(feature = "mock_close_proximity_disclosure")]
pub mod mock;
#[cfg(any(
    all(feature = "mock_close_proximity_disclosure", test),
    feature = "hardware_integration_test"
))]
pub mod test;

use async_trait::async_trait;
use tokio::sync::mpsc;

pub use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureChannel;
pub use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureError;
pub use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureUpdate;

#[cfg_attr(feature = "mock_close_proximity_disclosure", mockall::automock)]
pub trait CloseProximityDisclosureClient {
    async fn start_qr_handover()
    -> Result<(String, mpsc::Receiver<CloseProximityDisclosureUpdate>), CloseProximityDisclosureError>;

    async fn send_device_response(response: Vec<u8>) -> Result<(), CloseProximityDisclosureError>;

    async fn send_session_termination() -> Result<(), CloseProximityDisclosureError>;

    async fn stop_ble_server() -> Result<(), CloseProximityDisclosureError>;
}

pub struct CloseProximityDisclosureChannelImpl {
    sender: mpsc::Sender<CloseProximityDisclosureUpdate>,
}

impl CloseProximityDisclosureChannelImpl {
    pub fn new() -> (Self, mpsc::Receiver<CloseProximityDisclosureUpdate>) {
        let (sender, receiver) = mpsc::channel(128);
        (Self { sender }, receiver)
    }
}

#[async_trait]
impl CloseProximityDisclosureChannel for CloseProximityDisclosureChannelImpl {
    async fn send_update(&self, update: CloseProximityDisclosureUpdate) -> Result<(), CloseProximityDisclosureError> {
        self.sender
            .send(update)
            .await
            .map_err(|_| CloseProximityDisclosureError::PlatformError {
                reason: "channel receiver has been dropped".to_string(),
            })
    }
}
