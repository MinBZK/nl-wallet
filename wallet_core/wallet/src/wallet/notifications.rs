use std::pin::Pin;
use std::sync::Arc;

use tracing::info;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKeyHolder;
use utils::generator::TimeGenerator;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::NotificationType;
use crate::Wallet;
use crate::digid::DigidClient;
use crate::errors::StorageError;
use crate::notification::Notification;
use crate::repository::Repository;
use crate::storage::Storage;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum NotificationsError {
    #[error("could not fetch attestations from database storage: {0}")]
    Storage(#[from] StorageError),
}

pub type ScheduledNotificationsCallback = Box<dyn Fn(Vec<Notification>) + Send + Sync>;

type DirectNotificationFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub type DirectNotificationsCallback =
    Arc<dyn Fn(Vec<(i32, NotificationType)>) -> DirectNotificationFuture + Send + Sync>;

impl<CR, UR, S, AKH, APC, DC, IS, DCC, SLC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    pub async fn emit_notifications(&self) -> Result<(), NotificationsError> {
        info!("Emit notifications");

        let wallet_config = self.config_repository.get();
        let storage = self.storage.read().await;

        if let Some(notifications_callback) = &self.scheduled_notifications_callback {
            let notifications: Vec<Notification> = storage
                .fetch_unique_attestations()
                .await?
                .into_iter()
                .map(|copy| copy.into_attestation_presentation(&wallet_config.pid_attributes))
                .filter_map(|attestation| Notification::create_for_attestation(attestation, &TimeGenerator))
                .flatten()
                .collect();

            notifications_callback(notifications);
        }

        Ok(())
    }

    #[sentry_capture_error]
    pub async fn set_scheduled_notifications_callback(
        &mut self,
        callback: ScheduledNotificationsCallback,
    ) -> Result<Option<ScheduledNotificationsCallback>, NotificationsError> {
        let previous_callback = self.scheduled_notifications_callback.replace(callback);

        // If the `Wallet` is not registered, the database will not be open.
        // In that case don't emit anything.
        if self.registration.is_registered() {
            self.emit_notifications().await?;
        }

        Ok(previous_callback)
    }

    #[sentry_capture_error]
    pub fn set_direct_notifications_callback(
        &mut self,
        callback: DirectNotificationsCallback,
    ) -> Result<Option<DirectNotificationsCallback>, NotificationsError> {
        let previous_callback = self.direct_notifications_callback.lock().replace(callback);

        Ok(previous_callback)
    }

    pub fn clear_scheduled_notifications_callback(&mut self) {
        let callback = self.scheduled_notifications_callback.take();
        // Unschedule any existing notifications
        if let Some(callback) = callback {
            callback(Vec::new());
        }
    }

    pub fn clear_direct_notifications_callback(&mut self) {
        self.direct_notifications_callback.lock().take();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;
    use uuid::Uuid;

    use attestation_data::validity::ValidityWindow;

    use crate::Notification;
    use crate::storage::StoredAttestation;
    use crate::storage::StoredAttestationCopy;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::create_example_pid_sd_jwt;

    // Tests both setting and clearing the notifications callback on a `Wallet`.
    #[tokio::test]
    async fn test_wallet_set_clear_notifications_callback() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let (sd_jwt, sd_jwt_metadata) = create_example_pid_sd_jwt();

        let storage = wallet.mut_storage();
        storage.expect_fetch_unique_attestations().return_once(move || {
            Ok(vec![StoredAttestationCopy::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                ValidityWindow::new_valid_mock(),
                StoredAttestation::SdJwt {
                    key_identifier: "sd_jwt_key_id".to_string(),
                    sd_jwt,
                },
                sd_jwt_metadata,
                None,
            )])
        });

        let all_notifications: Arc<Mutex<Vec<Notification>>> = Arc::new(Mutex::new(Vec::with_capacity(2)));
        let callback_notifications = Arc::clone(&all_notifications);
        wallet
            .set_scheduled_notifications_callback(Box::new(move |notifications| {
                *callback_notifications.lock() = notifications;
            }))
            .await
            .unwrap();

        assert_eq!(all_notifications.lock().len(), 2);

        // Clear the notifications callback on the `Wallet` which also unschedules any existing notifications
        wallet.clear_scheduled_notifications_callback();
        assert!(all_notifications.lock().is_empty());
    }
}
