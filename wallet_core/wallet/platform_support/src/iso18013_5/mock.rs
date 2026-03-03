use crate::Iso18013_5Error;

use super::Iso18013_5SessionManager;

struct MockIso18013_5Session;

impl Iso18013_5SessionManager for MockIso18013_5Session {
    async fn start_qr_handover() -> Result<String, Iso18013_5Error> {
        Ok("some_qr_code".to_string())
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
