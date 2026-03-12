use tokio::sync::mpsc;

use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureChannel;
use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureError;
use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureUpdate;
use crate::close_proximity_disclosure::CloseProximityDisclosureChannelImpl;

use super::CloseProximityDisclosureClient;

pub struct MockCloseProximityDisclosureClient;

impl CloseProximityDisclosureClient for MockCloseProximityDisclosureClient {
    async fn start_qr_handover()
    -> Result<(String, mpsc::Receiver<CloseProximityDisclosureUpdate>), CloseProximityDisclosureError> {
        let (channel, receiver) = CloseProximityDisclosureChannelImpl::new();

        tokio::spawn(async move {
            let _ = channel.send_update(CloseProximityDisclosureUpdate::Connecting).await;

            let _ = channel.send_update(CloseProximityDisclosureUpdate::Connected).await;

            let _ = channel
                .send_update(CloseProximityDisclosureUpdate::SessionEstablished {
                    session_transcript: vec![0x01, 0x02, 0x03],
                    device_request: vec![0x04, 0x05, 0x06],
                })
                .await;

            let _ = channel.send_update(CloseProximityDisclosureUpdate::Closed).await;
        });

        Ok(("some_qr_code".to_string(), receiver))
    }

    async fn send_device_response(_response: Vec<u8>) -> Result<(), CloseProximityDisclosureError> {
        Ok(())
    }

    async fn stop_ble_server() -> Result<(), CloseProximityDisclosureError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test;
    use super::MockCloseProximityDisclosureClient;

    #[tokio::test]
    async fn test_mock_start_qr_handover() {
        test::test_start_qr_handover::<MockCloseProximityDisclosureClient>().await;
    }
}
