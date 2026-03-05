use tokio::sync::mpsc;

use crate::bridge::iso18013_5::Iso18013_5Channel;
use crate::bridge::iso18013_5::Iso18013_5Error;
use crate::bridge::iso18013_5::Iso18013_5Update;
use crate::iso18013_5::Iso18013_5ChannelImpl;

use super::Iso18013_5SessionManager;

struct MockIso18013_5Session;

impl Iso18013_5SessionManager for MockIso18013_5Session {
    async fn start_qr_handover() -> Result<(String, mpsc::Receiver<Iso18013_5Update>), Iso18013_5Error> {
        let (channel, receiver) = Iso18013_5ChannelImpl::new();

        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let _ = channel.send_update(Iso18013_5Update::Connecting).await;

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let _ = channel.send_update(Iso18013_5Update::Connected).await;

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let _ = channel
                .send_update(Iso18013_5Update::DeviceRequest {
                    session_transcript: vec![],
                    device_request: vec![],
                })
                .await;

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let _ = channel.send_update(Iso18013_5Update::Closed).await;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            drop(channel);
        });

        Ok(("some_qr_code".to_string(), receiver))
    }

    async fn send_device_response(_response: Vec<u8>) -> Result<(), Iso18013_5Error> {
        Ok(())
    }

    async fn stop_ble_server() -> Result<(), Iso18013_5Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test;
    use super::MockIso18013_5Session;

    #[tokio::test]
    async fn test_mock_start_qr_handover() {
        test::test_start_qr_handover::<MockIso18013_5Session>().await;
    }
}
