use indexmap::IndexMap;
use nutype::nutype;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::Entry;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::qualification::AttestationQualification;
use http_utils::urls::HttpsUri;

use crate::holder::MdocCredentialPayloadError;
use crate::DocType;
use crate::NameSpace;
use crate::Tdate;

#[nutype(
    derive(Debug, Clone, PartialEq, AsRef, TryFrom, Into, Serialize, Deserialize),
    validate(predicate = |attributes|
        !attributes.is_empty() && !attributes.values().any(|entries| entries.is_empty())
    ),
)]
pub struct UnsignedAttributes(IndexMap<NameSpace, Vec<Entry>>);

/// A collection of data representing a not-yet-signed mdoc.
#[derive(Debug, Clone)]
pub struct UnsignedMdoc {
    pub doc_type: DocType,
    pub valid_from: Tdate,
    pub valid_until: Tdate,
    pub attributes: UnsignedAttributes,

    /// The SAN DNS name or URI of the issuer, as it appears in the issuer's certificate.
    pub issuer_uri: HttpsUri,
    pub attestation_qualification: AttestationQualification,
}

impl TryFrom<PreviewableCredentialPayload> for UnsignedMdoc {
    type Error = MdocCredentialPayloadError;

    fn try_from(source: PreviewableCredentialPayload) -> Result<Self, Self::Error> {
        let attributes = Attribute::from_attributes(&source.attestation_type, source.attributes);

        let unsigned_mdoc = Self {
            doc_type: source.attestation_type,
            attributes: attributes
                .try_into()
                .map_err(|_| MdocCredentialPayloadError::NoAttributes)?,
            valid_from: source
                .not_before
                .ok_or(MdocCredentialPayloadError::MissingValidityTimestamp)?
                .into(),
            valid_until: source
                .expires
                .ok_or(MdocCredentialPayloadError::MissingValidityTimestamp)?
                .into(),
            issuer_uri: source.issuer,
            attestation_qualification: source.attestation_qualification,
        };

        Ok(unsigned_mdoc)
    }
}
