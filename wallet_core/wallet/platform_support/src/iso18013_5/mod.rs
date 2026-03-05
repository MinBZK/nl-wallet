pub mod hardware;

#[cfg(feature = "mock_iso18013_5")]
pub mod mock;
#[cfg(any(all(feature = "mock_iso18013_5", test), feature = "hardware_integration_test"))]
pub mod test;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::bridge::iso18013_5::Iso18013_5Channel;
use crate::bridge::iso18013_5::Iso18013_5Error;
use crate::bridge::iso18013_5::Iso18013_5Update;

pub trait Iso18013_5SessionManager {
    async fn start_qr_handover() -> Result<(String, mpsc::Receiver<Iso18013_5Update>), Iso18013_5Error>;

    async fn send_device_response(response: Vec<u8>) -> Result<(), Iso18013_5Error>;

    async fn stop_ble_server() -> Result<(), Iso18013_5Error>;
}

pub struct Iso18013_5ChannelImpl {
    sender: mpsc::Sender<Iso18013_5Update>,
}

impl Iso18013_5ChannelImpl {
    pub fn new() -> (Self, mpsc::Receiver<Iso18013_5Update>) {
        let (sender, receiver) = mpsc::channel(16);
        (Self { sender }, receiver)
    }
}

#[async_trait]
impl Iso18013_5Channel for Iso18013_5ChannelImpl {
    async fn send_update(&self, update: Iso18013_5Update) -> Result<(), Iso18013_5Error> {
        self.sender
            .send(update)
            .await
            .map_err(|_| Iso18013_5Error::PlatformError {
                reason: "channel receiver has been dropped".to_string(),
            })
    }
}
