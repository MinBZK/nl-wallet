use crate::pin::validate_pin;
use flutter_data_types::PinResult;

pub fn is_valid_pin(pin: String) -> Vec<u8> {
    let pin_result = PinResult::from(validate_pin(&pin));
    bincode::serialize(&pin_result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_is_valid_pin(pin: &str) -> bool {
        let serialized_pin_result = is_valid_pin(pin.to_owned());
        let pin_result = bincode::deserialize(&serialized_pin_result).unwrap();
        match pin_result {
            PinResult::Ok => true,
            PinResult::Err(_) => false,
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
