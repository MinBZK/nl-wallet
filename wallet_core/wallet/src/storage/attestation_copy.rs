use uuid::Uuid;

use attestation_data::credential_payload::CredentialPayload;
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

impl StoredAttestationCopy {
    pub fn into_credential_payload(self) -> CredentialPayload {
        match self.attestation {
            StoredAttestationFormat::MsoMdoc { mdoc } => {
                CredentialPayload::from_mdoc_unvalidated(*mdoc, &self.normalized_metadata)
                    .expect("conversion from stored mdoc attestation to CredentialPayload has been done before")
            }
            StoredAttestationFormat::SdJwt { sd_jwt } => CredentialPayload::from_verified_sd_jwt_unvalidated(*sd_jwt)
                .expect("conversion from stored SD-JWT attestation to CredentialPayload has been done before"),
        }
    }
}
