use crate::pin::validate_pin;

pub fn is_valid_pin(pin: String) -> bool {
    validate_pin(&pin).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_valid_pin() {
        assert!(is_valid_pin("142032".to_owned()));
    }

    #[test]
    fn check_invalid_pin() {
        assert!(!is_valid_pin("sdfioj".to_owned()));
    }
}
