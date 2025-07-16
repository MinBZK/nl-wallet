use error_category::ErrorCategory;
use error_category::sentry_capture_error;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(expected)]
pub enum PinValidationError {
    #[error("PIN contains characters that are not digits")]
    NonDigits,
    #[error("PIN does not have the required length")]
    InvalidLength,
    #[error("PIN has too few unique digits")]
    TooFewUniqueDigits,
    #[error("PIN digits in ascending order")]
    AscendingDigits,
    #[error("PIN digits are in descending order")]
    DescendingDigits,
}

// The expected length of the pin code
const EXACT_LENGTH: usize = 6;
// The minimum number of unique digits
const MIN_UNIQUE_DIGITS: usize = 2;
// The radix used to parse a digit, 10 for decimal (16 for hexadecimal)
const RADIX: usize = 10;

/// This function will check whether the pin has a valid length.
fn pin_length_should_be_correct(pin: &str) -> Result<(), PinValidationError> {
    if pin.len() != EXACT_LENGTH {
        Err(PinValidationError::InvalidLength)
    } else {
        Ok(())
    }
}

/// This function will convert a pin to a vector of digits.
/// It will return an error when non-digits are detected.
fn parse_pin_to_digits(pin: &str) -> Result<Vec<u8>, PinValidationError> {
    let digit_options: Vec<Option<u32>> = pin.chars().map(|c| c.to_digit(RADIX as u32)).collect();
    if digit_options.iter().any(|c| c.is_none()) {
        Err(PinValidationError::NonDigits)
    } else {
        let digits: Vec<u8> = digit_options.into_iter().map(|c| c.unwrap() as u8).collect();
        Ok(digits)
    }
}

/// This function will check whether there are enough unique digits.
// NOTE: this function will panic when the vector digits contains numbers > 9.
fn pin_should_contain_enough_unique_digits(digits: &[u8]) -> Result<(), PinValidationError> {
    let count: [u8; RADIX] = {
        let mut count: [u8; RADIX] = [0; RADIX];
        for d in digits {
            count[*d as usize] += 1;
        }
        count
    };
    let unique_digits = count.into_iter().filter(|&d| d > 0).count();
    if unique_digits < MIN_UNIQUE_DIGITS {
        Err(PinValidationError::TooFewUniqueDigits)
    } else {
        Ok(())
    }
}

/// This function will check whether the pin is ascending or descending
/// by a difference of 1 without modulo.
// NOTE: the assumption here is that a pin contains at least 2 digits.
// NOTE: this function will panic when an empty vector is supplied.
fn pin_should_not_be_ascending_or_descending(digits: &[u8]) -> Result<(), PinValidationError> {
    let mut ascending: bool = true;
    let mut descending: bool = true;
    let mut prev: i8 = digits[0] as i8;
    for &d in &digits[1..] {
        if (d as i8) != prev + 1 {
            ascending = false;
        }
        if (d as i8) != prev - 1 {
            descending = false;
        }
        prev = d as i8;
    }
    if ascending {
        Err(PinValidationError::AscendingDigits)
    } else if descending {
        Err(PinValidationError::DescendingDigits)
    } else {
        Ok(())
    }
}

/// This function will check whether a pin is not too simple.
#[sentry_capture_error]
pub fn validate_pin(pin: &str) -> Result<(), PinValidationError> {
    pin_length_should_be_correct(pin)?;
    let digits = parse_pin_to_digits(pin)?;
    pin_should_contain_enough_unique_digits(&digits)?;
    pin_should_not_be_ascending_or_descending(&digits)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pin() {
        assert!(matches!(validate_pin("024791"), Ok(())));
        assert!(matches!(validate_pin("010101"), Ok(())));
        assert!(matches!(validate_pin("000001"), Ok(())));
        assert!(matches!(validate_pin("100000"), Ok(())));
    }

    #[test]
    fn pin_should_have_length_6() {
        assert!(matches!(validate_pin("02479"), Err(PinValidationError::InvalidLength)));
        assert!(matches!(
            validate_pin("0247913"),
            Err(PinValidationError::InvalidLength)
        ));
    }

    #[test]
    fn pin_should_contain_only_digits() {
        assert!(matches!(validate_pin("abcdef"), Err(PinValidationError::NonDigits)));
        assert!(matches!(validate_pin("02479a"), Err(PinValidationError::NonDigits)));
    }

    #[test]
    fn pin_should_contain_at_least_2_unique_digits() {
        assert!(matches!(
            validate_pin("000000"),
            Err(PinValidationError::TooFewUniqueDigits)
        ));
        assert!(matches!(
            validate_pin("999999"),
            Err(PinValidationError::TooFewUniqueDigits)
        ));
    }

    #[test]
    fn pin_should_not_contain_ascending_digits() {
        assert!(matches!(
            validate_pin("012345"),
            Err(PinValidationError::AscendingDigits)
        ));
        assert!(matches!(
            validate_pin("456789"),
            Err(PinValidationError::AscendingDigits)
        ));
    }

    #[test]
    fn pin_should_not_contain_descending_digits() {
        assert!(matches!(
            validate_pin("543210"),
            Err(PinValidationError::DescendingDigits)
        ));
        assert!(matches!(
            validate_pin("987654"),
            Err(PinValidationError::DescendingDigits)
        ));
    }
}
