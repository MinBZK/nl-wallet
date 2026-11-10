use attestation_data::attributes::Attribute;
use attestation_data::attributes::Attributes;
use attestation_types::claim_path::ClaimPath;
use error_category::ErrorCategory;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::credential::CredentialWithMetadata;
use openid4vc::wallet_issuance::credential::IssuedCredentialCopies;
use platform_support::attested_key::AttestedKeyHolder;
use sd_jwt::claims::ClaimName;
use sd_jwt::claims::ClaimValue;
use sd_jwt::claims::ObjectClaims;
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
    AttestationQuery(#[source] StorageError),

    #[error("no PID received")]
    #[category(critical)]
    MissingPid,

    #[error("PID configuration error: {0}")]
    PidAttributesConfiguration(#[source] PidAttributesConfigurationError),
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
{
    /// Find the recovery code in the first SD-JWT PID among the newly issued credentials.
    pub(super) fn recovery_code_from_credentials<'a>(
        credentials: &'a [CredentialWithMetadata],
        pid_config: &PidAttributesConfiguration,
    ) -> Result<&'a str, RecoveryCodeError> {
        // Since the recovery code is the same in all copies, just find the first credential that is in SD-JWT format
        // and has one of the required `vct` values.
        let (pid_claims, sd_jwt) = credentials
            .iter()
            .find_map(|credential| match &credential.copies {
                IssuedCredentialCopies::SdJwt(copies) => pid_config
                    .sd_jwt
                    .get(&credential.contents.attestation_type)
                    .map(|pid_claims| (pid_claims, &copies.first().sd_jwt)),
                _ => None,
            })
            .ok_or(RecoveryCodeError::MissingPid)?;

        retrieve_recovery_code_from_decoded_claims(sd_jwt.decoded_claims(), &pid_claims.recovery_code_path)
            .ok_or(RecoveryCodeError::MissingIssuedRecoveryCode)
    }

    /// Check the received recovery code against the one in storage, if present.
    pub(super) async fn compare_recovery_code_against_stored(
        &self,
        received_recovery_code: &str,
        pid_config: &PidAttributesConfiguration,
    ) -> Result<(), RecoveryCodeError> {
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
                let recovery_code_path = pid_config
                    .sd_jwt_recovery_code_path(stored_copy.attestation_type())
                    .map_err(RecoveryCodeError::PidAttributesConfiguration)?;

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

fn retrieve_recovery_code_from_decoded_claims<'a>(
    decoded_claims: &'a ObjectClaims,
    recovery_code_path: &VecNonEmpty<ClaimName>,
) -> Option<&'a str> {
    let (first, rest) = recovery_code_path.split_first();

    let value = rest
        .iter()
        .try_fold(decoded_claims.get(first)?, |value, name| match value {
            ClaimValue::Object(object) => object.get(name),
            _ => None,
        })?;

    match value {
        ClaimValue::String(code) => Some(code.as_str()),
        _ => None,
    }
}

fn retrieve_recovery_code_from_pid<'a>(
    attributes: &'a Attributes,
    recovery_code_path: &VecNonEmpty<ClaimPath>,
) -> Option<&'a str> {
    attributes
        .get(recovery_code_path)
        .ok()
        .flatten()
        .and_then(|attribute| match attribute {
            Attribute::Text(code) => Some(code.as_str()),
            _ => None,
        })
}
