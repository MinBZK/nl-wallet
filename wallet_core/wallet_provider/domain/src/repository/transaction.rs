use tracing::warn;

use crate::repository::errors::PersistenceError;

pub trait Committable {
    async fn commit(self) -> Result<(), PersistenceError>;
}

pub trait TransactionStarter {
    type TransactionType: Committable;

    async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError>;
}

/// Perform [`operation`] and commit the [`transaction`] on error, otherwise return both the operation result and the
/// transaction. When committing the transaction fails, the commit error is logged and the original error is returned.
///
/// This is useful whenever any operation done before needs to be persisted even when this action fails.
/// This is an optimization for committing the previous transaction explicitly and starting a new transaction.
pub async fn commit_on_error<T, F, R, E>(transaction: T, operation: F) -> Result<(T, R), E>
where
    T: Committable,
    F: AsyncFnOnce(&T) -> Result<R, E>,
    E: From<PersistenceError>,
{
    let result = operation(&transaction).await;

    match result {
        Ok(result) => Ok((transaction, result)),
        Err(error) => {
            if let Err(commit_err) = transaction.commit().await {
                warn!("Failed to commit transaction after error: {commit_err}");
            }
            Err(error)
        }
    }
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
