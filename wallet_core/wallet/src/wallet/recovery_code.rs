use std::collections::HashSet;

use attestation_data::attributes::AttributeValue;
use attestation_types::claim_path::ClaimPath;
use dcql::CredentialFormat;
use error_category::ErrorCategory;
use openid4vc::Format;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::issuance_session::NormalizedCredentialPreview;
use platform_support::attested_key::AttestedKeyHolder;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use wallet_configuration::wallet_config::PidAttributesConfiguration;

use crate::digid::DigidClient;
use crate::errors::StorageError;
use crate::storage::Storage;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum RecoveryCodeError {
    #[error("no recovery code found in PID")]
    #[category(unexpected)]
    MissingRecoveryCode,

    #[error("incorrect recovery code: expected {expected}, received {received}")]
    #[category(pd)]
    IncorrectRecoveryCode {
        expected: AttributeValue,
        received: AttributeValue,
    },

    #[error("storage error: {0}")]
    #[category(unexpected)]
    Storage(#[from] StorageError),

    #[error("could not query attestations in database: {0}")]
    AttestationQuery(#[source] StorageError),

    #[error("cannot recover PIN without a PID")]
    #[category(critical)]
    NoPidPresent,

    #[error("no PID received")]
    #[category(unexpected)]
    MissingPid,
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    IS: IssuanceSession,
    DCC: DisclosureClient,
{
    pub(super) fn recovery_code_path(
        pid_config: &PidAttributesConfiguration,
        attestation_type: &str,
    ) -> VecNonEmpty<ClaimPath> {
        pid_config
            .sd_jwt
            .get(attestation_type)
            .expect("stored PID had no corresponding PID configuration")
            .recovery_code
            .nonempty_iter()
            .map(|path| ClaimPath::SelectByKey(path.to_string()))
            .collect()
    }

    pub(super) fn pid_preview<'a>(
        previews: &'a [NormalizedCredentialPreview],
        pid_config: &PidAttributesConfiguration,
    ) -> Result<&'a NormalizedCredentialPreview, RecoveryCodeError> {
        previews
            .iter()
            .find(|preview| {
                preview.content.copies_per_format.get(&Format::SdJwt).is_some()
                    && pid_config
                        .sd_jwt
                        .contains_key(&preview.content.credential_payload.attestation_type)
            })
            .ok_or(RecoveryCodeError::MissingPid)
    }

    /// Check the recovery code in the specified PID preview against the one in storage, if present.
    pub(super) async fn compare_recovery_code_against_stored(
        &self,
        pid_preview: &NormalizedCredentialPreview,
        pid_config: &PidAttributesConfiguration,
    ) -> Result<(), RecoveryCodeError> {
        let received_recovery_code = pid_preview
            .content
            .credential_payload
            .attributes
            .get(&Self::recovery_code_path(
                pid_config,
                &pid_preview.content.credential_payload.attestation_type,
            ))
            .expect("failed to retrieve recovery code from PID")
            .ok_or(RecoveryCodeError::MissingRecoveryCode)?
            .clone();

        let pid_attestation_types = pid_config.sd_jwt.keys().map(String::as_str).collect();
        let Some(stored_recovery_code) = self.stored_recovery_code(&pid_attestation_types, pid_config).await? else {
            return Ok(());
        };

        if stored_recovery_code != received_recovery_code {
            Err(RecoveryCodeError::IncorrectRecoveryCode {
                expected: stored_recovery_code.clone(),
                received: received_recovery_code,
            })
        } else {
            Ok(())
        }
    }

    async fn stored_recovery_code(
        &self,
        pid_attestation_types: &HashSet<&str>,
        pid_config: &PidAttributesConfiguration,
    ) -> Result<Option<AttributeValue>, RecoveryCodeError> {
        let recovery_code = self
            .storage
            .read()
            .await
            .fetch_unique_attestations_by_types_and_format(pid_attestation_types, CredentialFormat::SdJwt)
            .await
            .map_err(RecoveryCodeError::AttestationQuery)?
            .pop()
            .and_then(|stored| {
                let attestation_type = stored.attestation_type().to_string();

                stored
                    .into_attributes()
                    .get(&Self::recovery_code_path(pid_config, &attestation_type))
                    .expect("failed to retrieve recovery code from PID")
                    .cloned()
            });

        Ok(recovery_code)
    }
}
