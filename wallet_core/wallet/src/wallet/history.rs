use tracing::info;

use nl_wallet_mdoc::DocType;

use crate::{
    errors::StorageError,
    storage::{Storage, WalletEvent},
};

use super::Wallet;

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("wallet is locked")]
    Locked,
    #[error("could not access history database: {0}")]
    Storage(#[from] StorageError),
}

type Result<T> = std::result::Result<T, HistoryError>;

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    S: Storage,
{
    pub async fn get_history(&self) -> Result<Vec<WalletEvent>> {
        info!("Retrieving history");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(HistoryError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(HistoryError::Locked);
        }

        info!("Retrieving history from storage");
        let storage = self.storage.read().await;
        let events = storage.fetch_wallet_events().await?;
        Ok(events)
    }

    pub async fn get_history_for_card(&self, doc_type: &DocType) -> Result<Vec<WalletEvent>> {
        info!("Retrieving Card history");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(HistoryError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(HistoryError::Locked);
        }

        info!("Retrieving Card history from storage");
        let storage = self.storage.read().await;
        let events = storage.fetch_wallet_events_by_doc_type(doc_type).await?;
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use chrono::Utc;

    use nl_wallet_mdoc::utils::x509::Certificate;

    use crate::{storage::Storage, wallet::tests::WalletWithMocks, EventStatus, EventType, WalletEvent};

    use super::HistoryError;

    #[tokio::test]
    async fn test_history_fails_when_not_registered() {
        let wallet = WalletWithMocks::default();

        let error = wallet
            .get_history()
            .await
            .expect_err("Expect error when Wallet is not registered");
        assert_matches!(error, HistoryError::NotRegistered);

        let error = wallet
            .get_history_for_card(&"some-doc-type".to_owned())
            .await
            .expect_err("Expect error when Wallet is not registered");
        assert_matches!(error, HistoryError::NotRegistered);
    }

    #[tokio::test]
    async fn test_history_fails_when_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        let error = wallet
            .get_history()
            .await
            .expect_err("Expect error when Wallet is locked");
        assert_matches!(error, HistoryError::Locked);

        let error = wallet
            .get_history_for_card(&"some-doc-type".to_owned())
            .await
            .expect_err("Expect error when Wallet is locked");
        assert_matches!(error, HistoryError::Locked);
    }

    #[tokio::test]
    async fn test_history() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        let (certificate, _) = Certificate::new_ca("test-ca").unwrap();

        // history should be empty
        let history = wallet.get_history().await.unwrap();
        assert_eq!(history, vec![]);

        let some_doc_type = "some-doc-type".to_owned();
        let some_doc_type_event = WalletEvent::new(
            EventType::Issuance,
            some_doc_type.clone(),
            Utc::now(),
            certificate.clone(),
            EventStatus::Success,
        );
        let another_doc_type = "another-doc-type".to_owned();
        let another_doc_type_event = WalletEvent::new(
            EventType::Issuance,
            another_doc_type.clone(),
            Utc::now(),
            certificate,
            EventStatus::Success,
        );
        let events = vec![some_doc_type_event.clone(), another_doc_type_event.clone()];
        // log 2 history events
        wallet.storage.get_mut().log_wallet_events(events).await.unwrap();

        // get history should return both events
        let history = wallet.get_history().await.unwrap();
        assert_eq!(
            history,
            vec![some_doc_type_event.clone(), another_doc_type_event.clone()]
        );

        // get history for card should return single event
        let history = wallet.get_history_for_card(&some_doc_type).await.unwrap();
        assert_eq!(history, vec![some_doc_type_event]);

        let history = wallet.get_history_for_card(&another_doc_type).await.unwrap();
        assert_eq!(history, vec![another_doc_type_event]);
    }
}
