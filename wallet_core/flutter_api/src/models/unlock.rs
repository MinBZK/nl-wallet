use wallet::wallet::WalletUnlockError;

pub enum UnlockResult {
    Ok,
    IncorrectPin,
}

/// This conversion distinguishes between 3 distinct cases:
///
/// 1. In case of a successful result, [`UnlockResult::Ok`] will be returned.
/// 2. In case of an expected and/or specific error case a different variant of
///    [`UnlockResult`] will be returned.
/// 3. In any other cases, this is an unexpected and/or generic error and the
///    [`WalletUnlockError`] will be returned unchanged.
impl TryFrom<Result<(), WalletUnlockError>> for UnlockResult {
    // This is not currently used, but will be once more error variants are added.
    type Error = WalletUnlockError;

    fn try_from(value: Result<(), WalletUnlockError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(UnlockResult::Ok),
            Err(_) => Ok(UnlockResult::IncorrectPin),
        }
    }
}
