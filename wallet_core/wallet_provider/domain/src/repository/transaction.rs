use async_trait::async_trait;

use crate::repository::errors::PersistenceError;

#[async_trait]
pub trait Committable {
    async fn commit(self) -> Result<(), PersistenceError>;
}

#[async_trait]
pub trait TransactionStarter {
    type TransactionType: Committable;

    async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError>;
}

#[cfg(feature = "stub")]
pub mod stub {
    use super::*;

    pub struct TransactionStub;

    #[async_trait]
    impl Committable for TransactionStub {
        async fn commit(self) -> Result<(), PersistenceError> {
            Ok(())
        }
    }

    pub struct TransactionStarterStub;

    #[async_trait]
    impl TransactionStarter for TransactionStarterStub {
        type TransactionType = TransactionStub;

        async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
            Ok(TransactionStub {})
        }
    }
}
