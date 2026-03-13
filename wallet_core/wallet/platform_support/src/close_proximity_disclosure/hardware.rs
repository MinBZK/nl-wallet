use std::sync::Arc;

use tokio::sync::mpsc;

use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureError;
use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureUpdate;
use crate::bridge::close_proximity_disclosure::get_close_proximity_disclosure_bridge;
use crate::close_proximity_disclosure::CloseProximityDisclosureChannelImpl;
use crate::close_proximity_disclosure::CloseProximityDisclosureClient;

#[derive(Debug, Default)]
pub struct HardwareCloseProximityDisclosureClient;

impl CloseProximityDisclosureClient for HardwareCloseProximityDisclosureClient {
    async fn start_qr_handover()
    -> Result<(String, mpsc::Receiver<CloseProximityDisclosureUpdate>), CloseProximityDisclosureError> {
        let (channel, receiver) = CloseProximityDisclosureChannelImpl::new();
        let qr = get_close_proximity_disclosure_bridge()
            .start_qr_handover(Arc::new(channel))
            .await?;
        Ok((qr, receiver))
    }

    async fn send_device_response(response: Vec<u8>) -> Result<(), CloseProximityDisclosureError> {
        get_close_proximity_disclosure_bridge()
            .send_device_response(response)
            .await
    }

    async fn stop_ble_server() -> Result<(), CloseProximityDisclosureError> {
        get_close_proximity_disclosure_bridge().stop_ble_server().await
    }
}
