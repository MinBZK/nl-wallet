use tracing::info;

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

    pub async fn get_history_for_card(&self, doc_type: &str) -> Result<Vec<WalletEvent>> {
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

    use chrono::{TimeZone, Utc};
    use nl_wallet_mdoc::utils::x509::Certificate;
    use uuid::Uuid;

    use crate::{storage::Storage, wallet::tests::WalletWithMocks, EventStatus, EventType, WalletEvent};

    use super::HistoryError;

    #[tokio::test]
    async fn test_history_fails_when_not_registered() {
        let wallet = WalletWithMocks::new_unregistered().await;

        let error = wallet
            .get_history()
            .await
            .expect_err("Expect error when Wallet is not registered");
        assert_matches!(error, HistoryError::NotRegistered);

        let error = wallet
            .get_history_for_card("some-doc-type")
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
            .get_history_for_card("some-doc-type")
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

        let timestamp_older = Utc.with_ymd_and_hms(2023, 11, 11, 11, 11, 00).unwrap();
        let timestamp_newer = Utc.with_ymd_and_hms(2023, 11, 21, 13, 37, 00).unwrap();

        let some_doc_type = "some-doc-type";
        let some_doc_type_event = WalletEvent::new(
            Uuid::new_v4(),
            EventType::issuance_from_str(vec![some_doc_type]),
            timestamp_older,
            certificate.clone(),
            EventStatus::Success,
        );
        wallet
            .storage
            .get_mut()
            .log_wallet_event(some_doc_type_event.clone())
            .await
            .unwrap();

        let another_doc_type = "another-doc-type";
        let another_doc_type_event = WalletEvent::new(
            Uuid::new_v4(),
            EventType::issuance_from_str(vec![another_doc_type]),
            timestamp_newer,
            certificate,
            EventStatus::Success,
        );
        wallet
            .storage
            .get_mut()
            .log_wallet_event(another_doc_type_event.clone())
            .await
            .unwrap();

        // get history should return both events, in correct order, newest first
        let history = wallet.get_history().await.unwrap();
        assert_eq!(
            history,
            vec![another_doc_type_event.clone(), some_doc_type_event.clone()]
        );

        // get history for card should return single event
        let history = wallet.get_history_for_card(some_doc_type).await.unwrap();
        assert_eq!(history, vec![some_doc_type_event]);

        let history = wallet.get_history_for_card(another_doc_type).await.unwrap();
        assert_eq!(history, vec![another_doc_type_event]);
    }
}
