use anyhow::Result;

use wallet::{pin::validate_pin, WALLET};

use crate::models::pin::PinValidationResult;

pub fn is_valid_pin(pin: String) -> Vec<u8> {
    let pin_result = PinValidationResult::from(validate_pin(&pin));
    bincode::serialize(&pin_result).unwrap()
}

pub fn register(pin: String) -> Result<()> {
    // TODO return differentiated errors?
    WALLET.lock().expect("wallet lock failed").register(pin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_is_valid_pin(pin: &str) -> bool {
        let serialized_pin_result = is_valid_pin(pin.to_owned());
        let pin_result = bincode::deserialize(&serialized_pin_result).unwrap();
        match pin_result {
            PinValidationResult::Ok => true,
            _ => false,
        }
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
