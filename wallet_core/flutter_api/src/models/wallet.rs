use serde::{Deserialize, Serialize};

use wallet::wallet::WalletUnlockError;

impl From<Result<(), WalletUnlockError>> for WalletUnlockResult {
    fn from(source: Result<(), WalletUnlockError>) -> Self {
        match source {
            Ok(()) => WalletUnlockResult::Ok,
            Err(err) => match err {
                WalletUnlockError::IncorrectPin {
                    leftover_attempts,
                    is_final_attempt,
                } => WalletUnlockResult::IncorrectPin {
                    leftover_attempts,
                    is_final_attempt,
                },
                WalletUnlockError::ServerError => WalletUnlockResult::ServerError,
                WalletUnlockError::Timeout { timeout_millis } => WalletUnlockResult::Timeout { timeout_millis },
                WalletUnlockError::Blocked => WalletUnlockResult::Blocked,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WalletUnlockResult {
    Ok,
    IncorrectPin {
        leftover_attempts: u8,
        is_final_attempt: bool,
    },
    Timeout {
        timeout_millis: u32,
    },
    Blocked,
    ServerError,
}
