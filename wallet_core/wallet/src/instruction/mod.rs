mod client;
mod keys;

use error_category::ErrorCategory;
use jwt::error::JwtError;
use wallet_account::messages::errors::AccountError;

use crate::account_provider::AccountProviderError;
use crate::account_provider::AccountProviderResponseError;
use crate::storage::StorageError;

pub use self::client::InstructionClient;
pub use self::client::InstructionClientFactory;
pub use self::keys::RemoteEcdsaKey;
pub use self::keys::RemoteEcdsaKeyError;
pub use self::keys::RemoteEcdsaKeyFactory;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum InstructionError {
    #[error(
        "PIN provided is incorrect: (attempts_left_in_round: {attempts_left_in_round}, is_final_round: \
         {is_final_round})"
    )]
    #[category(expected)]
    IncorrectPin {
        attempts_left_in_round: u8,
        is_final_round: bool,
    },
    #[error("unlock disabled due to timeout")]
    #[category(expected)]
    Timeout { timeout_millis: u64 },
    #[error("unlock permanently disabled")]
    #[category(expected)]
    Blocked,
    #[error("server error: {0}")]
    ServerError(#[source] AccountProviderError),
    #[error("Wallet Provider could not validate instruction")]
    #[category(critical)]
    InstructionValidation,
    #[error("could not sign instruction: {0}")]
    Signing(#[source] wallet_account::error::EncodeError),
    #[error("could not validate instruction result received from Wallet Provider: {0}")]
    InstructionResultValidation(#[source] JwtError),
    #[error("could not store instruction sequence number in database: {0}")]
    StoreInstructionSequenceNumber(#[from] StorageError),
}

impl From<AccountProviderError> for InstructionError {
    fn from(value: AccountProviderError) -> Self {
        match value {
            AccountProviderError::Response(AccountProviderResponseError::Account(
                AccountError::IncorrectPin(data),
                _,
            )) => Self::IncorrectPin {
                attempts_left_in_round: data.attempts_left_in_round,
                is_final_round: data.is_final_round,
            },
            AccountProviderError::Response(AccountProviderResponseError::Account(
                AccountError::PinTimeout(data),
                _,
            )) => Self::Timeout {
                timeout_millis: data.time_left_in_ms,
            },
            AccountProviderError::Response(AccountProviderResponseError::Account(AccountError::AccountBlocked, _)) => {
                Self::Blocked
            }
            AccountProviderError::Response(AccountProviderResponseError::Account(
                AccountError::InstructionValidation,
                _,
            )) => Self::InstructionValidation,
            value => Self::ServerError(value),
        }
    }
}
