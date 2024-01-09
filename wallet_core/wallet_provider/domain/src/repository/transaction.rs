use crate::repository::errors::PersistenceError;

pub trait Committable {
    async fn commit(self) -> Result<(), PersistenceError>;
}

pub trait TransactionStarter {
    type TransactionType: Committable;

    async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError>;
}

#[cfg(feature = "mock")]
pub mod mock {
    use super::*;

    pub struct MockTransaction;

    impl Committable for MockTransaction {
        async fn commit(self) -> Result<(), PersistenceError> {
            Ok(())
        }
    }

    pub struct MockTransactionStarter;

    impl TransactionStarter for MockTransactionStarter {
        type TransactionType = MockTransaction;

        async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
            Ok(MockTransaction {})
        }
    }
}
