use crate::pin::change::ChangePinConfiguration;

impl ChangePinConfiguration for () {
    async fn max_retries(&self) -> u8 {
        3
    }
}
