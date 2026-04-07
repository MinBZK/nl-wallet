use tracing::warn;

use crate::repository::errors::PersistenceError;

pub trait Committable {
    async fn commit(self) -> Result<(), PersistenceError>;
}

pub trait TransactionStarter {
    type TransactionType: Committable;

    async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError>;
}

pub async fn commit_on_error<T, F, R, E>(tx: T, action: F) -> Result<(T, R), E>
where
    T: Committable,
    F: AsyncFnOnce(&T) -> Result<R, E>,
    E: From<PersistenceError>,
{
    let result = action(&tx).await;

    match result {
        Ok(result) => Ok((tx, result)),
        Err(error) => {
            if let Err(commit_err) = tx.commit().await {
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
