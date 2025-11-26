use std::collections::HashSet;

use attestation_data::attributes::AttributeValue;
use dcql::CredentialFormat;
use error_category::ErrorCategory;
use openid4vc::Format;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::issuance_session::NormalizedCredentialPreview;
use platform_support::attested_key::AttestedKeyHolder;
use wallet_configuration::wallet_config::PidAttributesConfiguration;
use wallet_configuration::wallet_config::PidAttributesConfigurationError;

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

    #[error("could not query attestations in database: {0}")]
    AttestationQuery(#[from] StorageError),

    #[error("no PID received")]
    #[category(unexpected)]
    MissingPid,

    #[error("PID configuration error: {0}")]
    PidAttributesConfiguration(#[from] PidAttributesConfigurationError),
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    IS: IssuanceSession,
    DCC: DisclosureClient,
{
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
            .get(&pid_config.recovery_code_path(&pid_preview.content.credential_payload.attestation_type)?)
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
        self.storage
            .read()
            .await
            .fetch_unique_attestations_by_types_and_format(pid_attestation_types, CredentialFormat::SdJwt)
            .await
            .map_err(RecoveryCodeError::AttestationQuery)?
            .pop()
            .map(|stored| {
                let attestation_type = stored.attestation_type().to_string();

                let value = stored
                    .into_attributes()
                    .get(&pid_config.recovery_code_path(&attestation_type)?)
                    .expect("failed to retrieve recovery code from PID")
                    .cloned();

                Ok(value)
            })
            .transpose()
            .map(Option::flatten)
    }
}
