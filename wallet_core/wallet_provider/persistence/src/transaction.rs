use async_trait::async_trait;
use sea_orm::{DatabaseTransaction, TransactionTrait};
use tokio::task;

use wallet_provider_domain::repository::{Committable, PersistenceError};

use crate::{database::Db, PersistenceConnection};

/// This wraps a [`DatabaseTransaction`] in an [`Option`], which should always
/// be present while the [`Transaction`] wrapper is alive. It will only ever be
/// [`None`] after the commit or rollback, which is beyond the lifetime of the
/// [`Transaction`].
pub struct Transaction(Option<DatabaseTransaction>);

impl Transaction {
    fn new(transaction: DatabaseTransaction) -> Self {
        Transaction(Option::from(transaction))
    }
}

impl PersistenceConnection<DatabaseTransaction> for Transaction {
    fn connection(&self) -> &DatabaseTransaction {
        self.0.as_ref().expect("Wrapped transaction no longer exists")
    }
}

#[async_trait]
impl Committable for Transaction {
    async fn commit(mut self) -> Result<(), PersistenceError> {
        self.0
            .take()
            .expect("Wrapped transaction no longer exists")
            .commit()
            .await
            .map_err(|e| PersistenceError::Transaction(e.into()))
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        let transaction = self.0.take();

        transaction.map(|t| {
            task::spawn(async move {
                // TODO: log any errors resulting from rollback
                let _ = t.rollback().await;
            })
        });
    }
}

pub async fn begin_transaction(db: &Db) -> Result<Transaction, PersistenceError> {
    db.connection()
        .begin()
        .await
        .map(Transaction::new)
        .map_err(|e| PersistenceError::Transaction(e.into()))
}
