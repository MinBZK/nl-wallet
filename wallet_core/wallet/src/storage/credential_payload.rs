use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::IntoCredentialPayload;

use crate::storage::StoredAttestationCopy;
use crate::storage::StoredAttestationFormat;

impl From<StoredAttestationCopy> for CredentialPayload {
    fn from(value: StoredAttestationCopy) -> Self {
        match value.attestation {
            StoredAttestationFormat::MsoMdoc { mdoc } => mdoc
                .into_credential_payload(&value.normalized_metadata)
                .expect("conversion to CredentialPayload has been done before"),
            StoredAttestationFormat::SdJwt { sd_jwt } => sd_jwt
                .into_inner()
                .into_credential_payload(&value.normalized_metadata)
                .expect("conversion to CredentialPayload has been done before"),
        }
    }
}
