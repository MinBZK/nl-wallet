mod client;
mod keys;

use wallet_common::{account::messages::errors::ErrorType, jwt::JwtError};

use crate::{
    account_provider::{AccountProviderError, AccountProviderResponseError},
    storage::StorageError,
};

pub use self::{
    client::InstructionClient,
    keys::{RemoteEcdsaKeyError, RemoteEcdsaKeyFactory},
};

#[derive(Debug, thiserror::Error)]
pub enum InstructionError {
    #[error(
        "PIN provided is incorrect: (attempts_left_in_round: {attempts_left_in_round}, is_final_round: {is_final_round})"
    )]
    IncorrectPin {
        attempts_left_in_round: u8,
        is_final_round: bool,
    },
    #[error("unlock disabled due to timeout")]
    Timeout { timeout_millis: u64 },
    #[error("unlock permanently disabled")]
    Blocked,
    #[error("server error: {0}")]
    ServerError(#[source] AccountProviderError),
    #[error("Wallet Provider could not validate instruction")]
    InstructionValidation,
    #[error("could not sign instruction: {0}")]
    Signing(#[source] wallet_common::account::errors::Error),
    #[error("could not validate instruction result received from Wallet Provider: {0}")]
    InstructionResultValidation(#[source] JwtError),
    #[error("could not store instruction sequence number in database: {0}")]
    StoreInstructionSequenceNumber(#[from] StorageError),
}

impl From<AccountProviderError> for InstructionError {
    fn from(value: AccountProviderError) -> Self {
        if let AccountProviderError::Response(AccountProviderResponseError::Data(_, errordata)) = &value {
            match errordata.typ {
                ErrorType::PinTimeout(data) => Self::Timeout {
                    timeout_millis: data.time_left_in_ms,
                },
                ErrorType::IncorrectPin(data) => Self::IncorrectPin {
                    attempts_left_in_round: data.attempts_left_in_round,
                    is_final_round: data.is_final_round,
                },
                ErrorType::AccountBlocked => Self::Blocked,
                ErrorType::InstructionValidation => Self::InstructionValidation,
                _ => Self::ServerError(value),
            }
        } else {
            Self::ServerError(value)
        }
    }
}
