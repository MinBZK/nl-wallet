use attestation_data::attributes::Attribute;
use attestation_data::attributes::Attributes;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_types::claim_path::ClaimPath;
use attestation_types::credential_format::Format;
use error_category::ErrorCategory;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::OfferedCredentialPreview;
use platform_support::attested_key::AttestedKeyHolder;
use utils::vec_at_least::VecNonEmpty;
use wallet_configuration::wallet_config::PidAttributesConfiguration;
use wallet_configuration::wallet_config::PidAttributesConfigurationError;

use super::Wallet;
use crate::errors::StorageError;
use crate::storage::Storage;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum RecoveryCodeError {
    #[error("no recovery code found in newly issued PID")]
    #[category(critical)]
    MissingIssuedRecoveryCode,

    #[error("no recovery code found in stored PID")]
    #[category(impossible)]
    MissingStoredRecoveryCode,

    #[error("incorrect recovery code: expected {expected}, received {received}")]
    #[category(pd)]
    IncorrectRecoveryCode { expected: String, received: String },

    #[error("could not query attestations in database: {0}")]
    AttestationQuery(#[from] StorageError),

    #[error("no PID received")]
    #[category(critical)]
    MissingPid,

    #[error("PID configuration error: {0}")]
    PidAttributesConfiguration(#[from] PidAttributesConfigurationError),
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
{
    pub(super) fn pid_preview<'a>(
        mut previews: impl Iterator<Item = OfferedCredentialPreview<'a>>,
        pid_config: &PidAttributesConfiguration,
    ) -> Result<&'a PreviewableCredentialPayload, RecoveryCodeError> {
        // Find the first preview that is in SD-JWT format and has one of the required `vct` values. In theory there
        // could be more credentials in the preview that match, but we assume the caller just needs a single PID
        // preview, so simply ignore any subsequent matches.
        previews
            .find(|preview| {
                preview.format == Format::SdJwt
                    && pid_config
                        .sd_jwt
                        .contains_key(&preview.credential_payload.attestation_type)
            })
            .map(|preview| preview.credential_payload)
            .ok_or(RecoveryCodeError::MissingPid)
    }

    /// Check the recovery code in the specified PID preview against the one in storage, if present.
    pub(super) async fn compare_recovery_code_against_stored(
        &self,
        pid_preview: &PreviewableCredentialPayload,
        pid_config: &PidAttributesConfiguration,
    ) -> Result<(), RecoveryCodeError> {
        let received_recovery_code = retrieve_recovery_code_from_pid(
            &pid_preview.attributes,
            &pid_config.recovery_code_path(&pid_preview.attestation_type)?,
        )
        .ok_or(RecoveryCodeError::MissingIssuedRecoveryCode)?;

        // Fetch the the recovery code of the previously issued PID, return early if this is the first time we are
        // receiving the PID.
        let Some(stored_recovery_code) = self.stored_recovery_code(pid_config).await? else {
            return Ok(());
        };

        if stored_recovery_code != received_recovery_code {
            Err(RecoveryCodeError::IncorrectRecoveryCode {
                expected: stored_recovery_code,
                received: received_recovery_code.to_string(),
            })
        } else {
            Ok(())
        }
    }

    async fn stored_recovery_code(
        &self,
        pid_config: &PidAttributesConfiguration,
    ) -> Result<Option<String>, RecoveryCodeError> {
        // The recovery code is only present in the PID in the SD-JWT format.
        let code = self
            .storage
            .read()
            .await
            .fetch_unique_attestations_by_credential_kinds(&pid_config.sd_jwt_credential_kinds())
            .await
            .map_err(RecoveryCodeError::AttestationQuery)?
            .pop()
            .map(|stored_copy| -> Result<_, RecoveryCodeError> {
                let recovery_code_path = pid_config.recovery_code_path(stored_copy.attestation_type())?;

                // The stored PID has to include a recovery code, since this was checked during issuance. However, it is
                // still possible that the SD-JWT credential kinds or path to the recovery code was changed in the
                // configuration. This should never happen, making this an error in the "impossible" category.
                let code = retrieve_recovery_code_from_pid(&stored_copy.into_attributes(), &recovery_code_path)
                    .ok_or(RecoveryCodeError::MissingStoredRecoveryCode)?
                    .to_string();

                Ok(code)
            })
            .transpose()?;

        Ok(code)
    }
}

fn retrieve_recovery_code_from_pid<'a>(
    attributes: &'a Attributes,
    recovery_code_path: &VecNonEmpty<ClaimPath>,
) -> Option<&'a str> {
    attributes
        .get(recovery_code_path)
        .expect("failed to retrieve recovery code from PID")
        .and_then(|attribute| match attribute {
            Attribute::Text(code) => Some(code.as_str()),
            _ => None,
        })
}
