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

#[cfg(feature = "mock")]
pub mod mock {
    use super::*;

    pub struct MockTransaction;

    #[async_trait]
    impl Committable for MockTransaction {
        async fn commit(self) -> Result<(), PersistenceError> {
            Ok(())
        }
    }

    pub struct MockTransactionStarter;

    #[async_trait]
    impl TransactionStarter for MockTransactionStarter {
        type TransactionType = MockTransaction;

        async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
            Ok(MockTransaction {})
        }
    }
}
