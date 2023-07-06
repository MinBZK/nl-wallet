use async_trait::async_trait;

use crate::model::wallet_user::WalletUserCreate;

use super::{errors::PersistenceError, transaction::Committable};

#[async_trait]
pub trait WalletUserRepository {
    type TransactionType: Committable;

    /// Create a wallet user in the database.
    async fn create_wallet_user(
        &self,
        transaction: &Self::TransactionType,
        user: WalletUserCreate,
    ) -> Result<(), PersistenceError>;
}

#[cfg(feature = "stub")]
pub mod stub {
    use super::{super::transaction::stub::TransactionStub, *};

    pub struct WalletUserRepositoryStub;

    #[async_trait]
    impl WalletUserRepository for WalletUserRepositoryStub {
        type TransactionType = TransactionStub;

        async fn create_wallet_user(
            &self,
            _transaction: &Self::TransactionType,
            _user: WalletUserCreate,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }
    }
}
