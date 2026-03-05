use std::sync::Arc;

use tokio::sync::mpsc;

use crate::bridge::iso18013_5::Iso18013_5Error;
use crate::bridge::iso18013_5::Iso18013_5Update;
use crate::bridge::iso18013_5::get_iso18013_5_bridge;
use crate::iso18013_5::Iso18013_5ChannelImpl;
use crate::iso18013_5::Iso18013_5SessionManager;

pub struct HardwareIso18013_5SessionManager;

impl Iso18013_5SessionManager for HardwareIso18013_5SessionManager {
    async fn start_qr_handover() -> Result<(String, mpsc::Receiver<Iso18013_5Update>), Iso18013_5Error> {
        let (channel, receiver) = Iso18013_5ChannelImpl::new();
        let qr = get_iso18013_5_bridge().start_qr_handover(Arc::new(channel)).await?;
        Ok((qr, receiver))
    }

    async fn send_device_response(response: Vec<u8>) -> Result<(), Iso18013_5Error> {
        get_iso18013_5_bridge().send_device_response(response).await
    }

    async fn stop_ble_server() -> Result<(), Iso18013_5Error> {
        get_iso18013_5_bridge().stop_ble_server().await
    }
}
