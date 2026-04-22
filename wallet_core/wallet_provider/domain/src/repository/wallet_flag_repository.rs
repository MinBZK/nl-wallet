use super::errors::PersistenceError;
use crate::model::wallet_flag::WalletFlag;

type Result<T> = std::result::Result<T, PersistenceError>;

#[trait_variant::make(Send)]
pub trait WalletFlagRepository {
    async fn fetch_flags(&self) -> Result<Vec<(WalletFlag, bool)>>;

    async fn get_flag(&self, flag: WalletFlag) -> Result<bool>;

    async fn set_flag(&self, flag: WalletFlag) -> Result<()>;

    async fn clear_flag(&self, flag: WalletFlag) -> Result<()>;
}
