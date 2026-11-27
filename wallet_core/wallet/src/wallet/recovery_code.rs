use attestation_types::credential_kind::CredentialKindRef;
use error_category::ErrorCategory;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::credential::CredentialWithMetadata;
use openid4vc::wallet_issuance::credential::IssuedCredentialCopies;
use platform_support::attested_key::AttestedKeyHolder;
use sd_jwt::claims::ClaimName;
use sd_jwt::claims::ClaimValue;
use sd_jwt::claims::ObjectClaims;
use sd_jwt::error::ClaimError;
use sd_jwt::sd_jwt::UnverifiedSdJwt;
use utils::vec_at_least::VecNonEmpty;
use wallet_configuration::wallet_config::PidAttributesConfiguration;

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

    #[error("could not add recovery code disclosure: {0}")]
    #[category(pd)]
    RecoveryCodeDisclosure(#[source] ClaimError),

    #[error("could not query attestations in database: {0}")]
    AttestationQuery(#[source] StorageError),

    #[error("no PID received")]
    #[category(critical)]
    MissingPid,
}

/// A disclosure of just the recovery code of the issued PID, whose recovery code matches that of the stored PID, if
/// any.
#[derive(Debug, Clone, derive_more::Into)]
pub(super) struct CheckedRecoveryCodeDisclosure(UnverifiedSdJwt);

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
{
    /// Find the recovery code in the first SD-JWT PID among the issued credentials, check it against the recovery code
    /// of the stored PID, if any, and create a disclosure of just that claim for the Wallet Provider.
    ///
    /// Disclosing the recovery code of another person, e.g. when someone else logged in at the PID issuer, would make
    /// the Wallet Provider revoke all wallets of both persons.
    pub(super) async fn checked_recovery_code_disclosure(
        &self,
        credentials: &VecNonEmpty<CredentialWithMetadata>,
        pid_config: &PidAttributesConfiguration,
    ) -> Result<CheckedRecoveryCodeDisclosure, RecoveryCodeError> {
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

        let recovery_code =
            retrieve_recovery_code_from_decoded_claims(sd_jwt.decoded_claims(), &pid_claims.recovery_code_path)
                .ok_or(RecoveryCodeError::MissingIssuedRecoveryCode)?;

        self.compare_recovery_code_against_stored(recovery_code, pid_config)
            .await?;

        let disclosure = sd_jwt
            .clone()
            .into_sd_jwt()
            .into_presentation_builder()
            .disclose(&pid_claims.recovery_code_claim_path())
            .map_err(RecoveryCodeError::RecoveryCodeDisclosure)?
            .finish()
            .into();

        Ok(CheckedRecoveryCodeDisclosure(disclosure))
    }

    /// Check the received recovery code against the one in storage, if present.
    async fn compare_recovery_code_against_stored(
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
            .fetch_unique_attestations_by_credential_kinds(
                pid_config
                    .sd_jwt_credential_kinds()
                    .map(CredentialKindRef::into_owned)
                    .collect(),
            )
            .await
            .map_err(RecoveryCodeError::AttestationQuery)?
            .pop()
            .map(|stored_copy| {
                // The stored PID was checked for a recovery code during issuance, so this can only fail if the
                // PID configuration changed since, which should never happen.
                pid_config
                    .sd_jwt
                    .get(stored_copy.attestation_type())
                    .zip(stored_copy.sd_jwt_decoded_claims())
                    .and_then(|(pid_claims, claims)| {
                        retrieve_recovery_code_from_decoded_claims(claims, &pid_claims.recovery_code_path)
                    })
                    .map(ToOwned::to_owned)
                    .ok_or(RecoveryCodeError::MissingStoredRecoveryCode)
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
