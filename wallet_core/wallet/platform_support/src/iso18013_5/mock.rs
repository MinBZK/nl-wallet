use tokio::sync::mpsc;

use crate::bridge::iso18013_5::Iso18013_5Error;
use crate::bridge::iso18013_5::Iso18013_5Update;

use super::Iso18013_5SessionManager;

struct MockIso18013_5Session;

impl Iso18013_5SessionManager for MockIso18013_5Session {
    async fn start_qr_handover() -> Result<(String, mpsc::Receiver<Iso18013_5Update>), Iso18013_5Error> {
        let (_sender, receiver) = mpsc::channel(1);
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
