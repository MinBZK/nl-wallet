use tracing::info;

use error_category::sentry_capture_error;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKeyHolder;

use crate::Wallet;
use crate::digid::DigidClient;
use crate::errors::StorageError;
use crate::notification::Notification;
use crate::storage::Storage;

pub type NotificationsCallback = Box<dyn FnMut(Vec<Notification>) + Send + Sync>;

impl<CR, UR, S, AKH, APC, DC, IS, DCC, SLC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC, SLC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    pub async fn emit_notifications(&mut self) -> Result<(), StorageError> {
        info!("Emit recent history from storage");

        if let Some(notifications_callback) = &mut self.notifications_callback {
            let _storage = self.storage.read().await;

            notifications_callback(vec![]);
        }

        Ok(())
    }

    #[sentry_capture_error]
    pub async fn set_notifications_callback(
        &mut self,
        callback: NotificationsCallback,
    ) -> Result<Option<NotificationsCallback>, StorageError> {
        let previous_callback = self.notifications_callback.replace(Box::new(callback));

        Ok(previous_callback)
    }

    pub fn clear_notifications_callback(&mut self) {
        self.notifications_callback.take();
    }
}
