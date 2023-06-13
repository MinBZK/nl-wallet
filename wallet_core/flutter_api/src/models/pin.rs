use wallet::PinValidationError;

pub enum PinValidation {
    Ok,
    TooFewUniqueDigits,
    SequentialDigits,
    OtherIssue,
}

impl From<Result<(), PinValidationError>> for PinValidation {
    fn from(value: Result<(), PinValidationError>) -> Self {
        match value {
            Ok(_) => Self::Ok,
            Err(e) => match e {
                PinValidationError::NonDigits => Self::OtherIssue,
                PinValidationError::InvalidLength => Self::OtherIssue,
                PinValidationError::TooFewUniqueDigits => Self::TooFewUniqueDigits,
                PinValidationError::AscendingDigits => Self::SequentialDigits,
                PinValidationError::DescendingDigits => Self::SequentialDigits,
            },
        }
    }
}
