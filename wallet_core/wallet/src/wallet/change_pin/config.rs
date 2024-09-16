use crate::{wallet::change_pin::ChangePinConfiguration, Wallet};

impl ChangePinConfiguration for Wallet {
    async fn max_retries(&self) -> u8 {
        3
    }
}
