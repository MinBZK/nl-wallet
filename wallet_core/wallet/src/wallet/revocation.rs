use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use futures::StreamExt;
use rustls_pki_types::TrustAnchor;
use tokio::sync::RwLock;
use tokio::time;
use tokio::time::MissedTickBehavior;
use tracing::error;
use tracing::instrument;
use uuid::Uuid;

use error_category::ErrorCategory;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::IssuanceSession;
use platform_support::attested_key::AttestedKeyHolder;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationStatus;
use token_status_list::verification::verifier::RevocationVerifier;
use update_policy_model::update_policy::VersionState;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::Wallet;
use crate::account_provider::AccountProviderClient;
use crate::digid::DigidClient;
use crate::errors::StorageError;
use crate::repository::Repository;
use crate::storage::RevocationInfo;
use crate::storage::Storage;
use crate::wallet::attestations::AttestationsError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum RevocationError {
    #[error("error emitting attestations: {0}")]
    #[category(unexpected)]
    Attestations(#[from] AttestationsError),

    #[error("storage error: {0}")]
    #[category(unexpected)]
    Storage(#[from] StorageError),
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC, SLC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    IS: IssuanceSession,
    DCC: DisclosureClient,
    APC: AccountProviderClient,
    SLC: StatusListClient,
{
    /// Start background revocation checks
    ///
    /// Spawns a background task that only accesses storage, not the entire wallet.
    /// Stores the ablort handle in the wallet.
    #[instrument(skip_all)]
    pub fn start_background_revocation_checks(&mut self, check_interval: Duration)
    where
        S: Send + Sync + 'static,
        SLC: Send + Sync + 'static,
    {
        // Clone only what is needed for the background task
        let config = self.config_repository.get();
        let status_list_client = Arc::clone(&self.status_list_client);
        let storage = Arc::clone(&self.storage);

        let task = tokio::spawn(async move {
            let mut interval = time::interval(check_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                interval.tick().await;

                let issuer_trust_anchors = config.issuer_trust_anchors();

                if let Err(e) = Self::check_revocations(
                    &issuer_trust_anchors,
                    Arc::clone(&status_list_client),
                    Arc::clone(&storage),
                    &TimeGenerator,
                )
                .await
                {
                    error!("Background revocation check failed: {}", e);
                }
            }
        });

        // Store the abort handle
        self.revocation_status_job_handle = Some(task.abort_handle());
    }

    /// Perform revocation checks where all revocation info is fetched from storage
    async fn check_revocations(
        issuer_trust_anchors: &[TrustAnchor<'_>],
        status_list_client: Arc<SLC>,
        storage: Arc<RwLock<S>>,
        time_generator: &impl Generator<DateTime<Utc>>,
    ) -> Result<(), RevocationError>
    where
        SLC: StatusListClient,
        S: Storage,
    {
        let revocation_verifier = RevocationVerifier::new(Arc::clone(&status_list_client));

        // Fetch revocation info in one storage lock
        let revocation_info = storage.read().await.fetch_all_revocation_info().await?;

        // Verify all revocations without holding any locks
        let updates: Vec<(Uuid, RevocationStatus)> = futures::stream::iter(revocation_info)
            .map(|revocation_info| {
                Self::check_revocation(
                    revocation_info,
                    &revocation_verifier,
                    issuer_trust_anchors,
                    time_generator,
                )
            })
            .buffer_unordered(10)
            .collect()
            .await;

        // Write all updates in one storage lock
        storage.write().await.update_revocation_statuses(updates).await?;

        Ok(())
    }

    /// Perform revocation check using revocation info of a single attestation
    async fn check_revocation(
        revocation_info: RevocationInfo,
        revocation_verifier: &RevocationVerifier<SLC>,
        issuer_trust_anchors: &[TrustAnchor<'_>],
        time_generator: &impl Generator<DateTime<Utc>>,
    ) -> (Uuid, RevocationStatus)
    where
        SLC: StatusListClient,
    {
        let status = revocation_info
            .verify_revocation(issuer_trust_anchors, revocation_verifier, time_generator)
            .await;

        (revocation_info.attestation_copy_id(), status)
    }
}
