use wallet::IssuanceResult;
use wallet::errors::ChangePinError;
use wallet::errors::DisclosureBasedIssuanceError;
use wallet::errors::DisclosureError;
use wallet::errors::InstructionError;
use wallet::errors::IssuanceError;
use wallet::errors::PinRecoveryError;
use wallet::errors::TransferError;
use wallet::errors::WalletUnlockError;

use super::attestation::AttestationPresentation;

pub enum WalletInstructionResult {
    Ok,
    InstructionError { error: WalletInstructionError },
}

pub enum PidIssuanceResult {
    Ok { transfer_available: bool },
    InstructionError { error: WalletInstructionError },
}

pub enum DisclosureBasedIssuanceResult {
    Ok(Vec<AttestationPresentation>),
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
impl TryFrom<Result<IssuanceResult, IssuanceError>> for WalletInstructionResult {
    type Error = IssuanceError;

    fn try_from(value: Result<IssuanceResult, IssuanceError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(WalletInstructionResult::Ok),
            Err(IssuanceError::Instruction(instruction_error)) => Ok(WalletInstructionResult::InstructionError {
                error: instruction_error.try_into().map_err(IssuanceError::Instruction)?,
            }),
            Err(error) => Err(error),
        }
    }
}

// TODO remove this
impl TryFrom<Result<(), PinRecoveryError>> for WalletInstructionResult {
    type Error = PinRecoveryError;

    fn try_from(value: Result<(), PinRecoveryError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(WalletInstructionResult::Ok),
            Err(error) => Err(error),
        }
    }
}

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`PidIssuanceResult::Ok`] will be returned.
/// 2. In case of an expected and/or specific error case a different variant of [`PidIssuanceResult`] by mapping the
///    nested [InstructionError].
/// 3. In any other cases, this is an unexpected and/or generic error and the [`PidIssuanceError`] will be returned
///    unchanged.
impl TryFrom<Result<IssuanceResult, IssuanceError>> for PidIssuanceResult {
    type Error = IssuanceError;

    fn try_from(value: Result<IssuanceResult, IssuanceError>) -> Result<Self, Self::Error> {
        match value {
            Ok(IssuanceResult { transfer_session_id }) => Ok(PidIssuanceResult::Ok {
                transfer_available: transfer_session_id.is_some(),
            }),
            Err(IssuanceError::Instruction(instruction_error)) => Ok(PidIssuanceResult::InstructionError {
                error: instruction_error.try_into().map_err(IssuanceError::Instruction)?,
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

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`DisclosureBasedIssuanceResult::Ok`] will be returned, with the attestations
///    converted into the expected format.
/// 2. In case of an expected and/or specific error case a different variant of [`WalletInstructionResult`] by mapping
///    the nested [InstructionError].
/// 3. In any other cases, this is an unexpected and/or generic error and the [`DisclosureBasedIssuanceError`] will be
///    returned unchanged.
impl TryFrom<Result<Vec<wallet::AttestationPresentation>, DisclosureBasedIssuanceError>>
    for DisclosureBasedIssuanceResult
{
    type Error = DisclosureBasedIssuanceError;

    fn try_from(
        value: Result<Vec<wallet::AttestationPresentation>, DisclosureBasedIssuanceError>,
    ) -> Result<Self, Self::Error> {
        match value {
            Ok(attestations) => Ok(Self::Ok(
                attestations.into_iter().map(AttestationPresentation::from).collect(),
            )),
            Err(DisclosureBasedIssuanceError::Disclosure(DisclosureError::Instruction(instruction_error))) => {
                Ok(DisclosureBasedIssuanceResult::InstructionError {
                    error: instruction_error.try_into().map_err(|error| {
                        DisclosureBasedIssuanceError::Disclosure(DisclosureError::Instruction(error))
                    })?,
                })
            }
            Err(DisclosureBasedIssuanceError::Issuance(IssuanceError::Instruction(instruction_error))) => {
                Ok(DisclosureBasedIssuanceResult::InstructionError {
                    error: instruction_error
                        .try_into()
                        .map_err(|error| DisclosureBasedIssuanceError::Issuance(IssuanceError::Instruction(error)))?,
                })
            }
            Err(error) => Err(error),
        }
    }
}

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`WalletInstructionResult::Ok`] will be returned.
/// 2. In case of an expected and/or specific error case a different variant of [`WalletInstructionResult`] by mapping
///    the nested [InstructionError].
/// 3. In any other cases, this is an unexpected and/or generic error and the [`TransferError`] will be returned
///    unchanged.
impl TryFrom<Result<(), TransferError>> for WalletInstructionResult {
    type Error = TransferError;

    fn try_from(value: Result<(), TransferError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(WalletInstructionResult::Ok),
            Err(TransferError::Instruction(instruction_error)) => Ok(WalletInstructionResult::InstructionError {
                error: instruction_error.try_into().map_err(TransferError::Instruction)?,
            }),
            Err(error) => Err(error),
        }
    }
}
