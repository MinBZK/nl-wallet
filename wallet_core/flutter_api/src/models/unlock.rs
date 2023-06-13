use wallet::wallet::WalletUnlockError;

use crate::errors::FlutterApiError;

pub enum UnlockResult {
    Ok,
    IncorrectPin,
}

impl TryFrom<Result<(), WalletUnlockError>> for UnlockResult {
    // This is not currently used, but will be once more error variants are added.
    type Error = FlutterApiError;

    fn try_from(value: Result<(), WalletUnlockError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => Ok(UnlockResult::Ok),
            Err(_) => Ok(UnlockResult::IncorrectPin),
        }
    }
}
