use mdoc::holder::Mdoc;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;
use utils::date_time_seconds::DateTimeSeconds;
use utils::vec_at_least::VecNonEmpty;

#[derive(Clone, Debug)]
pub struct CredentialWithMetadata {
    pub copies: IssuedCredentialCopies,
    pub attestation_type: String,
    pub expiration: Option<DateTimeSeconds>,
    pub not_before: Option<DateTimeSeconds>,
    pub extended_attestation_types: Vec<String>,
    pub metadata_documents: VerifiedTypeMetadataDocuments,
}

impl CredentialWithMetadata {
    pub fn new(
        copies: IssuedCredentialCopies,
        attestation_type: String,
        expiration: Option<DateTimeSeconds>,
        not_before: Option<DateTimeSeconds>,
        extended_attestation_types: impl IntoIterator<Item = impl Into<String>>,
        metadata_documents: VerifiedTypeMetadataDocuments,
    ) -> Self {
        Self {
            copies,
            attestation_type,
            expiration,
            not_before,
            extended_attestation_types: extended_attestation_types.into_iter().map(Into::into).collect(),
            metadata_documents,
        }
    }
}

#[derive(Clone, Debug)]
pub enum IssuedCredentialCopies {
    Mdoc(VecNonEmpty<Mdoc>),
    SdJwt(VecNonEmpty<(String, VerifiedSdJwt)>),
}
