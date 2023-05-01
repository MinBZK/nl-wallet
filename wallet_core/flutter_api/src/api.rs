use anyhow::Result;

use flutter_api_macros::async_runtime;
use wallet::pin::validation::validate_pin;

use crate::{models::pin::PinValidationResult, wallet::WALLET};

pub fn init_async() {
    crate::async_runtime::try_init_async().expect("CORE may only be initialized once.");
}

pub fn is_valid_pin(pin: String) -> Vec<u8> {
    let pin_result = PinValidationResult::from(validate_pin(&pin));
    bincode::serialize(&pin_result).unwrap()
}

#[async_runtime]
pub async fn register(pin: String) -> Result<()> {
    // TODO return differentiated errors?
    WALLET.lock().await.register(pin).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_is_valid_pin(pin: &str) -> bool {
        let serialized_pin_result = is_valid_pin(pin.to_owned());
        let pin_result = bincode::deserialize(&serialized_pin_result).unwrap();
        matches!(pin_result, PinValidationResult::Ok)
    }

    #[test]
    fn check_valid_pin() {
        assert!(test_is_valid_pin("142032"));
    }

    #[test]
    fn check_invalid_pin() {
        assert!(!test_is_valid_pin("sdfioj"));
    }
}
