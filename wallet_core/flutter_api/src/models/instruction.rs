use wallet::{errors::InstructionError, wallet::PidIssuanceError, wallet::WalletUnlockError};

pub enum WalletInstructionResult {
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

/// This converts the [InstructionError] to the corresponding [WalletInstructionResult].
/// If no matching [WalletInstructionResult] is available the [InstructionError] will be returned
/// unchanged.
impl TryFrom<InstructionError> for WalletInstructionResult {
    type Error = InstructionError;

    fn try_from(value: InstructionError) -> Result<Self, Self::Error> {
        match value {
            InstructionError::IncorrectPin {
                leftover_attempts,
                is_final_attempt,
            } => Ok(WalletInstructionResult::IncorrectPin {
                leftover_attempts,
                is_final_attempt,
            }),
            InstructionError::Timeout { timeout_millis } => Ok(WalletInstructionResult::Timeout { timeout_millis }),
            InstructionError::Blocked => Ok(WalletInstructionResult::Blocked),
            _ => Err(value),
        }
    }
}

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`WalletInstructionResult::Ok`] will be returned.
/// 2. In case of an expected and/or specific error case a different variant of
///    [`WalletInstructionResult`] by converting the nested [InstructionError].
/// 3. In any other cases, this is an unexpected and/or generic error and the
///    [`WalletUnlockError`] will be returned unchanged.
impl TryFrom<Result<(), WalletUnlockError>> for WalletInstructionResult {
    type Error = WalletUnlockError;

    fn try_from(value: Result<(), WalletUnlockError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(WalletInstructionResult::Ok),
            Err(e) => match e {
                WalletUnlockError::Instruction(i) => Ok(i.try_into()?),
                _ => Err(e),
            },
        }
    }
}

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`WalletInstructionResult::Ok`] will be returned.
/// 2. In case of an expected and/or specific error case a different variant of
///    [`WalletInstructionResult`] by mapping the nested [InstructionError].
/// 3. In any other cases, this is an unexpected and/or generic error and the
///    [`PidIssuanceError`] will be returned unchanged.
impl TryFrom<Result<(), PidIssuanceError>> for WalletInstructionResult {
    type Error = PidIssuanceError;

    fn try_from(value: Result<(), PidIssuanceError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(WalletInstructionResult::Ok),
            Err(e) => match e {
                PidIssuanceError::Instruction(i) => Ok(i.try_into()?),
                _ => Err(e),
            },
        }
    }
}
