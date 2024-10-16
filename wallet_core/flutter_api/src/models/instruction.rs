use wallet::errors::{ChangePinError, InstructionError, PidIssuanceError, WalletUnlockError};

pub enum WalletInstructionResult {
    Ok,
    InstructionError { error: WalletInstructionError },
}

pub enum WalletInstructionError {
    IncorrectPin {
        attempts_left_in_round: u8,
        is_final_round: bool,
    },
    Timeout {
        timeout_millis: u64,
    },
    Blocked,
}

/// This converts the [InstructionError] to the corresponding [WalletInstructionResult].
/// If no matching [WalletInstructionResult] is available the [InstructionError] will be returned
/// unchanged.
impl TryFrom<InstructionError> for WalletInstructionError {
    type Error = InstructionError;

    fn try_from(value: InstructionError) -> Result<Self, Self::Error> {
        match value {
            InstructionError::IncorrectPin {
                attempts_left_in_round,
                is_final_round,
            } => Ok(WalletInstructionError::IncorrectPin {
                attempts_left_in_round,
                is_final_round,
            }),
            InstructionError::Timeout { timeout_millis } => Ok(WalletInstructionError::Timeout { timeout_millis }),
            InstructionError::Blocked => Ok(WalletInstructionError::Blocked),
            _ => Err(value),
        }
    }
}

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`WalletInstructionResult::Ok`] will be returned.
/// 2. In case of an expected and/or specific error case a different variant of [`WalletInstructionResult`] by
///    converting the nested [InstructionError].
/// 3. In any other cases, this is an unexpected and/or generic error and the [`WalletUnlockError`] will be returned
///    unchanged.
impl TryFrom<Result<(), WalletUnlockError>> for WalletInstructionResult {
    type Error = WalletUnlockError;

    fn try_from(value: Result<(), WalletUnlockError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(WalletInstructionResult::Ok),
            Err(WalletUnlockError::Instruction(instruction_error)) => Ok(WalletInstructionResult::InstructionError {
                error: instruction_error.try_into().map_err(WalletUnlockError::Instruction)?,
            }),
            Err(error) => Err(error),
        }
    }
}

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`WalletInstructionResult::Ok`] will be returned.
/// 2. In case of an expected and/or specific error case a different variant of [`WalletInstructionResult`] by mapping
///    the nested [InstructionError].
/// 3. In any other cases, this is an unexpected and/or generic error and the [`PidIssuanceError`] will be returned
///    unchanged.
impl TryFrom<Result<(), PidIssuanceError>> for WalletInstructionResult {
    type Error = PidIssuanceError;

    fn try_from(value: Result<(), PidIssuanceError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(WalletInstructionResult::Ok),
            Err(PidIssuanceError::Instruction(instruction_error)) => Ok(WalletInstructionResult::InstructionError {
                error: instruction_error.try_into().map_err(PidIssuanceError::Instruction)?,
            }),
            Err(error) => Err(error),
        }
    }
}

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`WalletInstructionResult::Ok`] will be returned.
/// 2. In case of an expected and/or specific error case a different variant of [`WalletInstructionResult`] by mapping
///    the nested [InstructionError].
/// 3. In any other cases, this is an unexpected and/or generic error and the [`ChangePinError`] will be returned
///    unchanged.
impl TryFrom<Result<(), ChangePinError>> for WalletInstructionResult {
    type Error = ChangePinError;

    fn try_from(value: Result<(), ChangePinError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(WalletInstructionResult::Ok),
            Err(ChangePinError::Instruction(instruction_error)) => Ok(WalletInstructionResult::InstructionError {
                error: instruction_error.try_into().map_err(ChangePinError::Instruction)?,
            }),
            Err(error) => Err(error),
        }
    }
}
