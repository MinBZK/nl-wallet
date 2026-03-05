use crate::model::wallet_flag::WalletFlag;

use super::errors::PersistenceError;

type Result<T> = std::result::Result<T, PersistenceError>;

#[trait_variant::make(Send)]
pub trait WalletFlagRepository {
    async fn fetch_flags(&self) -> Result<Vec<(WalletFlag, bool)>>;

    async fn set_flag(&self, flag: WalletFlag) -> Result<()>;

    async fn clear_flag(&self, flag: WalletFlag) -> Result<()>;
}
