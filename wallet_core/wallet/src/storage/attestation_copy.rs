use uuid::Uuid;

use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::IntoCredentialPayload;
use mdoc::holder::Mdoc;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;

#[derive(Debug, Clone)]
pub enum StoredAttestationFormat {
    MsoMdoc { mdoc: Box<Mdoc> }, // TODO: Wrap in similar VerifiedMdoc type (PVW-4132)
    SdJwt { sd_jwt: Box<VerifiedSdJwt> },
}

#[derive(Debug, Clone)]
pub struct StoredAttestationCopy {
    pub attestation_id: Uuid,
    pub attestation_copy_id: Uuid,
    pub attestation: StoredAttestationFormat,
    pub normalized_metadata: NormalizedTypeMetadata,
}

impl From<StoredAttestationCopy> for CredentialPayload {
    fn from(value: StoredAttestationCopy) -> Self {
        match value.attestation {
            StoredAttestationFormat::MsoMdoc { mdoc } => mdoc
                .into_credential_payload(&value.normalized_metadata)
                .expect("conversion from mdoc to CredentialPayload has been done before"),
            StoredAttestationFormat::SdJwt { sd_jwt } => sd_jwt
                .into_inner()
                .into_credential_payload(&value.normalized_metadata)
                .expect("conversion from SD-JWT to CredentialPayload has been done before"),
        }
    }
}
