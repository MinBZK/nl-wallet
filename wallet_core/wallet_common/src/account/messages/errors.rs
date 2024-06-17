use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumDiscriminants)]
#[strum_discriminants(
    name(AccountErrorType),
    derive(strum::Display, strum::EnumString),
    strum(serialize_all = "snake_case")
)]
pub enum AccountError {
    Unexpected,
    ChallengeValidation,
    RegistrationParsing,
    IncorrectPin(IncorrectPinData),
    PinTimeout(PinTimeoutData),
    AccountBlocked,
    InstructionValidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncorrectPinData {
    pub attempts_left_in_round: u8,
    pub is_final_round: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinTimeoutData {
    pub time_left_in_ms: u64,
}

impl From<AccountError> for Map<String, Value> {
    fn from(value: AccountError) -> Self {
        match value {
            AccountError::IncorrectPin(data) => serde_json::to_value(data).into(),
            AccountError::PinTimeout(data) => serde_json::to_value(data).into(),
            _ => None,
        }
        .transpose()
        .expect("AccountError data should serialize")
        .map(|value| {
            let Value::Object(map) = value else {
                panic!("AccountError data should be an object")
            };

            map
        })
        .unwrap_or_default()
    }
}

impl AccountError {
    pub fn try_from_type_and_data(
        r#type: AccountErrorType,
        data: Map<String, Value>,
    ) -> Result<Self, serde_json::Error> {
        let data = Value::Object(data);

        let account_error = match r#type {
            AccountErrorType::Unexpected => Self::Unexpected,
            AccountErrorType::ChallengeValidation => Self::ChallengeValidation,
            AccountErrorType::RegistrationParsing => Self::RegistrationParsing,
            AccountErrorType::IncorrectPin => Self::IncorrectPin(serde_json::from_value(data)?),
            AccountErrorType::PinTimeout => Self::PinTimeout(serde_json::from_value(data)?),
            AccountErrorType::AccountBlocked => Self::AccountBlocked,
            AccountErrorType::InstructionValidation => Self::InstructionValidation,
        };

        Ok(account_error)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_account_error_conversion() {
        let error = AccountError::IncorrectPin(IncorrectPinData {
            attempts_left_in_round: 2,
            is_final_round: false,
        });

        let error_type = AccountErrorType::from(&error);
        let error_data = error.into();

        let parsed_error =
            AccountError::try_from_type_and_data(error_type, error_data).expect("should parse successfully");

        assert_matches!(
            parsed_error,
            AccountError::IncorrectPin(IncorrectPinData {
                attempts_left_in_round,
                is_final_round
            }) if attempts_left_in_round == 2 && !is_final_round
        )
    }
}
