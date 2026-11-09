use attestation_data::credential_payload::CredentialContents;
use attestation_types::credential_format::Format;
use attestation_types::credential_kind::CredentialKindRef;
use crypto::x509::BorrowingCertificate;
use mdoc::holder::Mdoc;
use nonempty_collections::NEEither;
use nonempty_collections::NonEmptyIterator;
use sd_jwt::sd_jwt::DecodedSdJwt;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;
use utils::vec_at_least::VecNonEmpty;

use super::issuer_registration::IssuerRegistration;
use crate::metadata::issuer_metadata::CredentialMetadata;

/// The metadata that describes an issued credential.
#[derive(Clone, Debug)]
pub enum IssuedCredentialMetadata {
    TypeMetadata {
        normalized: NormalizedTypeMetadata,
        verified: VerifiedTypeMetadataDocuments,
    },
    CredentialMetadata(CredentialMetadata),
}

#[cfg(any(test, feature = "mock"))]
impl IssuedCredentialMetadata {
    pub fn nl_pid_example() -> Self {
        Self::TypeMetadata {
            normalized: NormalizedTypeMetadata::nl_pid_example(),
            verified: VerifiedTypeMetadataDocuments::nl_pid_example(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CredentialWithMetadata {
    pub copies: IssuedCredentialCopies,
    /// The contents of the first copy, which is identical for all copies.
    pub contents: CredentialContents,
    pub extended_attestation_types: Vec<String>,
    pub metadata: IssuedCredentialMetadata,
    pub issuer_registration: IssuerRegistration,
}

impl CredentialWithMetadata {
    /// The combination of the format of the issued copies and the attestation type of this credential.
    pub fn credential_kind(&self) -> CredentialKindRef<'_> {
        CredentialKindRef::new(self.copies.format(), &self.contents.attestation_type)
    }

    pub fn new(
        copies: IssuedCredentialCopies,
        contents: CredentialContents,
        extended_attestation_types: impl IntoIterator<Item = impl Into<String>>,
        metadata: IssuedCredentialMetadata,
        issuer_registration: IssuerRegistration,
    ) -> Self {
        Self {
            copies,
            contents,
            extended_attestation_types: extended_attestation_types.into_iter().map(Into::into).collect(),
            metadata,
            issuer_registration,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MdocCopy {
    pub key_identifier: String,
    pub mdoc: Mdoc,
}

#[derive(Debug, Clone)]
pub struct SdJwtCopy {
    pub key_identifier: String,
    pub sd_jwt: DecodedSdJwt,
}

#[derive(Debug, Clone)]
pub enum IssuedCredentialCopies {
    Mdoc(VecNonEmpty<MdocCopy>),
    SdJwt(VecNonEmpty<SdJwtCopy>),
}

impl IssuedCredentialCopies {
    pub fn format(&self) -> Format {
        match self {
            IssuedCredentialCopies::Mdoc(_) => Format::MsoMdoc,
            IssuedCredentialCopies::SdJwt(_) => Format::SdJwt,
        }
    }

    pub fn issuer_certificate_chains(&self) -> impl NonEmptyIterator<Item = &VecNonEmpty<BorrowingCertificate>> {
        match self {
            IssuedCredentialCopies::Mdoc(copies) => {
                NEEither::Left(copies.nonempty_iter().map(|copy| copy.mdoc.issuer_certificate_chain()))
            }
            IssuedCredentialCopies::SdJwt(copies) => NEEither::Right(
                copies
                    .nonempty_iter()
                    .map(|copy| copy.sd_jwt.sd_jwt().issuer_certificate_chain()),
            ),
        }
    }
}
