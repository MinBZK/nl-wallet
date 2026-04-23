use derive_more::AsRef;
use derive_more::Constructor;
use mdoc::holder::Mdoc;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;
use utils::date_time_seconds::DateTimeSeconds;
use utils::vec_at_least::VecNonEmpty;

#[derive(Clone, Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "in practice, variants are less different in size"
)]
pub enum IssuedCredential {
    MsoMdoc {
        mdoc: Mdoc,
    },
    SdJwt {
        // This uniquely identifies the holder private key used for this credential, as managed by the WSCD.
        key_identifier: String,
        sd_jwt: VerifiedSdJwt,
    },
}

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

#[derive(Clone, Debug, AsRef, Constructor)]
pub struct IssuedCredentialCopies(VecNonEmpty<IssuedCredential>);

impl IssuedCredentialCopies {
    pub fn into_inner(self) -> VecNonEmpty<IssuedCredential> {
        self.0
    }
}

#[cfg(feature = "example_constructors")]
mod example_constructors {
    use std::mem;

    use utils::vec_at_least::VecNonEmpty;

    use super::IssuedCredential;
    use super::IssuedCredentialCopies;

    impl IssuedCredentialCopies {
        pub fn new_or_panic(copies: VecNonEmpty<IssuedCredential>) -> Self {
            let first = copies.first();
            if copies
                .iter()
                .any(|credential| mem::discriminant(credential) != mem::discriminant(first))
            {
                panic!("different credential format encountered in issued copies");
            }

            Self(copies)
        }
    }
}
