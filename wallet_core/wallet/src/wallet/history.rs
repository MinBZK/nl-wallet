use crate::{
    errors::StorageError,
    storage::{Storage, WalletEvent},
};

use super::Wallet;

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("could not access history database: {0}")]
    Storage(#[from] StorageError),
}

type Result<T> = std::result::Result<T, HistoryError>;

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    S: Storage,
{
    pub async fn get_history(&self) -> Result<Vec<WalletEvent>> {
        let storage = self.storage.read().await;
        let events = storage.fetch_wallet_events().await?;
        Ok(events)
    }

    pub async fn get_history_for_card(&self, doc_type: String) -> Result<Vec<WalletEvent>> {
        let storage = self.storage.read().await;
        let events = storage.fetch_wallet_events_by_doc_type(doc_type).await?;
        Ok(events)
    }
}
