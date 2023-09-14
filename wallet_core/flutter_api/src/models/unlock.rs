use wallet::errors::{InstructionError, WalletUnlockError};

pub enum WalletUnlockResult {
    Ok,
    IncorrectPin {
        leftover_attempts: u8,
        is_final_attempt: bool,
    },
    Timeout {
        timeout_millis: u64,
    },
    Blocked,
}

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`WalletUnlockResult::Ok`] will be returned.
/// 2. In case of an expected and/or specific error case a different variant of
///    [`WalletUnlockResult`] will be returned.
/// 3. In any other cases, this is an unexpected and/or generic error and the
///    [`WalletUnlockError`] will be returned unchanged.
impl TryFrom<Result<(), WalletUnlockError>> for WalletUnlockResult {
    // This is not currently used, but will be once more error variants are added.
    type Error = WalletUnlockError;

    fn try_from(value: Result<(), WalletUnlockError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(WalletUnlockResult::Ok),
            Err(e) => match e {
                WalletUnlockError::Instruction(InstructionError::IncorrectPin {
                    leftover_attempts,
                    is_final_attempt,
                }) => Ok(WalletUnlockResult::IncorrectPin {
                    leftover_attempts,
                    is_final_attempt,
                }),
                WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis }) => {
                    Ok(WalletUnlockResult::Timeout { timeout_millis })
                }
                WalletUnlockError::Instruction(InstructionError::Blocked) => Ok(WalletUnlockResult::Blocked),
                _ => Err(e),
            },
        }
    }
}
