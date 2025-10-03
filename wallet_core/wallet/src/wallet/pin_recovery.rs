use std::sync::Arc;

use tracing::info;
use url::Url;

use attestation_data::constants::PID_ATTESTATION_TYPE;
use attestation_data::constants::PID_RECOVERY_CODE;
use attestation_types::claim_path::ClaimPath;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::IssuanceSession;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use utils::vec_nonempty;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::AttestationAttributeValue;
use crate::account_provider::AccountProviderClient;
use crate::digid::DigidClient;
use crate::digid::DigidSession;
use crate::repository::Repository;
use crate::storage::AttestationFormatQuery;
use crate::storage::Storage;
use crate::wallet::IssuanceError;
use crate::wallet::Session;

use super::Wallet;

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    IS: IssuanceSession,
    DCC: DisclosureClient,
    APC: AccountProviderClient,
{
    pub async fn create_pin_recovery_redirect_uri(&mut self) -> Result<Url, IssuanceError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(IssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(IssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(IssuanceError::Locked);
        }

        info!("Checking if there is an active session");
        if self.session.is_some() {
            return Err(IssuanceError::SessionState);
        }

        self.pid_issuance_auth_url().await
    }

    pub async fn continue_pin_recovery(&mut self, redirect_uri: Url) -> Result<(), IssuanceError> {
        info!("Received redirect URI, processing URI and retrieving access token");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(IssuanceError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(IssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(IssuanceError::Locked);
        }

        if !matches!(self.session, Some(Session::Digid(..))) {
            return Err(IssuanceError::SessionState);
        }

        info!("Checking if there is an active DigiD issuance session");
        let Some(Session::Digid(session)) = self.session.take() else {
            return Err(IssuanceError::SessionState);
        };

        let pid_issuance_config = &self.config_repository.get().pid_issuance;
        let token_request = session
            .into_token_request(&pid_issuance_config.digid_http_config, redirect_uri)
            .await
            .map_err(IssuanceError::DigidSessionFinish)?;

        let config = self.config_repository.get();

        // Check the recovery code in the received PID against the one in the stored PID, as otherwise
        // the WP will reject our PIN recovery instructions.

        let previews = self
            .issuance_fetch_previews(
                token_request,
                config.pid_issuance.pid_issuer_url.clone(),
                &config.issuer_trust_anchors(),
                true,
            )
            .await?;

        let received_recovery_code = &previews
            .first()
            .unwrap()
            .attributes
            .iter()
            .find(|attr| attr.key == vec![PID_RECOVERY_CODE])
            .expect("TODO")
            .value;

        let AttestationAttributeValue::Basic(received_recovery_code) = received_recovery_code else {
            panic!("TODO");
        };

        let stored_pid_credential_payload = self
            .storage
            .write()
            .await
            .fetch_unique_attestations_by_type(&[PID_ATTESTATION_TYPE].into(), AttestationFormatQuery::SdJwt)
            .await
            .map_err(IssuanceError::AttestationQuery)?
            .pop()
            .expect("no PID found in registered wallet")
            .into_credential_payload();

        let stored_recovery_code = stored_pid_credential_payload
            .previewable_payload
            .attributes
            .get(&vec_nonempty![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_string())])
            .expect("failed to retrieve recovery code from PID")
            .expect("no recovery code found in PID");

        if stored_recovery_code != received_recovery_code {
            panic!("TODO")
        }

        Ok(())
    }
}
