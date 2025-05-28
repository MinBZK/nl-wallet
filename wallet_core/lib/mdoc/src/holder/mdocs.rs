use std::result::Result;

use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use indexmap::IndexSet;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use ssri::Integrity;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeError;
use attestation_data::attributes::Entry;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::IntoCredentialPayload;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::identifiers::AttributeIdentifier;
use crypto::keys::CredentialEcdsaKey;
use crypto::keys::CredentialKeyType;
use crypto::x509::BorrowingCertificate;
use error_category::ErrorCategory;
use jwt::error::JwkConversionError;
use jwt::jwk::jwk_from_p256;
use sd_jwt::key_binding_jwt_claims::RequiredKeyBinding;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataValidationError;
use utils::generator::Generator;

use crate::errors::Error;
use crate::iso::*;
use crate::utils::cose::CoseError;
use crate::utils::crypto::CryptoError;
use crate::verifier::ValidityRequirement;

use super::HolderError;

/// A full mdoc: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mdoc {
    /// Mobile Security Object of the mdoc. This is also present inside the `issuer_signed`; we include it here for
    /// convenience (fetching it from the `issuer_signed` would involve parsing the COSE inside it).
    pub mso: MobileSecurityObject,

    /// Identifier of the mdoc's private key. Obtain a reference to it with [`Keyfactory::generate(private_key_id)`].
    // Note that even though these fields are not `pub`, to users of this package their data is still accessible
    // by serializing the mdoc and examining the serialized bytes. This is not a problem because it is essentially
    // unavoidable: when stored (i.e. serialized), we need to include all of this data to be able to recover a usable
    // mdoc after deserialization.
    pub private_key_id: String,
    pub issuer_signed: IssuerSigned,
    key_type: CredentialKeyType,
}

impl Mdoc {
    /// Construct a new `Mdoc`, verifying it against the specified thrust anchors before returning it.
    pub fn new<K: CredentialEcdsaKey>(
        private_key_id: String,
        issuer_signed: IssuerSigned,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> crate::Result<Mdoc> {
        let (_, mso) = issuer_signed.verify(ValidityRequirement::AllowNotYetValid, time, trust_anchors)?;
        let mdoc = Mdoc {
            mso,
            private_key_id,
            issuer_signed,
            key_type: K::KEY_TYPE,
        };
        Ok(mdoc)
    }

    pub fn issuer_certificate(&self) -> Result<BorrowingCertificate, CoseError> {
        self.issuer_signed.issuer_auth.signing_cert()
    }

    pub fn issuer_signed_attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.issuer_signed.attribute_identifiers(self.doc_type())
    }

    pub fn doc_type(&self) -> &String {
        &self.mso.doc_type
    }

    pub fn type_metadata_integrity(&self) -> Result<&Integrity, Error> {
        let integrity = self
            .mso
            .type_metadata_integrity
            .as_ref()
            .ok_or(HolderError::MissingMetadataIntegrity)?;

        Ok(integrity)
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum MdocCredentialPayloadError {
    #[error("error converting to / from JSON: {0}")]
    #[category(pd)]
    JsonConversion(#[from] serde_json::Error),

    #[error("metadata validation error: {0}")]
    #[category(pd)]
    MetadataValidation(#[from] TypeMetadataValidationError),

    #[error("unable to convert mdoc TDate to DateTime<Utc>")]
    #[category(critical)]
    DateConversion(#[from] chrono::ParseError),

    #[error("mdoc is missing issuer URI")]
    #[category(critical)]
    MissingIssuerUri,

    #[error("mdoc is missing attestation qualification")]
    #[category(critical)]
    MissingAttestationQualification,

    #[error("attribute error: {0}")]
    #[category(pd)]
    Attribute(#[from] AttributeError),

    #[error("error converting holder VerifyingKey to JWK: {0}")]
    #[category(pd)]
    JwkConversion(#[from] JwkConversionError),

    #[error("error converting holder public CoseKey to a VerifyingKey: {0}")]
    #[category(pd)]
    CoseKeyConversion(#[from] CryptoError),

    #[error("no attributes present in PreviewableCredentialPayload")]
    #[category(critical)]
    NoAttributes,

    #[error("missing either the \"exp\" or \"nbf\" timestamp")]
    #[category(critical)]
    MissingValidityTimestamp,
}

impl IntoCredentialPayload for Mdoc {
    type Error = MdocCredentialPayloadError;

    fn into_credential_payload(self, metadata: &NormalizedTypeMetadata) -> Result<CredentialPayload, Self::Error> {
        MdocParts::new(self.issuer_signed.into_entries_by_namespace(), self.mso).into_credential_payload(metadata)
    }
}

#[derive(derive_more::Constructor)]
pub struct MdocParts {
    attributes: IndexMap<NameSpace, Vec<Entry>>,
    mso: MobileSecurityObject,
}

impl IntoCredentialPayload for MdocParts {
    type Error = MdocCredentialPayloadError;

    fn into_credential_payload(self, metadata: &NormalizedTypeMetadata) -> Result<CredentialPayload, Self::Error> {
        let MdocParts { attributes, mso } = self;

        let holder_pub_key = VerifyingKey::try_from(mso.device_key_info)?;

        let payload = CredentialPayload {
            issued_at: (&mso.validity_info.signed).try_into()?,
            confirmation_key: jwk_from_p256(&holder_pub_key).map(RequiredKeyBinding::Jwk)?,
            previewable_payload: PreviewableCredentialPayload {
                attestation_type: mso.doc_type,
                issuer: mso.issuer_uri.ok_or(MdocCredentialPayloadError::MissingIssuerUri)?,
                expires: Some((&mso.validity_info.valid_until).try_into()?),
                not_before: Some((&mso.validity_info.valid_from).try_into()?),
                attestation_qualification: mso
                    .attestation_qualification
                    .ok_or(MdocCredentialPayloadError::MissingAttestationQualification)?,
                attributes: Attribute::from_mdoc_attributes(metadata, attributes)?,
            },
        };

        CredentialPayload::validate(&serde_json::to_value(&payload)?, metadata)?;

        Ok(payload)
    }
}

#[cfg(any(test, feature = "test"))]
mod test {
    use p256::ecdsa::VerifyingKey;
    use ssri::Integrity;

    use crypto::server_keys::KeyPair;
    use crypto::CredentialEcdsaKey;
    use crypto::EcdsaKey;

    use crate::iso::disclosure::IssuerSigned;
    use crate::iso::mdocs::IssuerSignedItemBytes;
    use crate::iso::unsigned::UnsignedMdoc;

    use super::Mdoc;

    impl Mdoc {
        /// Construct an [`Mdoc`] directly by signing, skipping validation.
        pub async fn sign<K: CredentialEcdsaKey>(
            unsigned_mdoc: UnsignedMdoc,
            metadata_integrity: Integrity,
            private_key_id: String,
            public_key: &VerifyingKey,
            issuer_keypair: &KeyPair<impl EcdsaKey>,
        ) -> crate::Result<Mdoc> {
            let (issuer_signed, mso) =
                IssuerSigned::sign(unsigned_mdoc, metadata_integrity, public_key, issuer_keypair).await?;

            let mdoc = Self {
                mso,
                private_key_id,
                issuer_signed,
                key_type: K::KEY_TYPE,
            };

            Ok(mdoc)
        }

        pub fn modify_attributes<F>(&mut self, name_space: &str, modify_func: F)
        where
            F: FnOnce(&mut Vec<IssuerSignedItemBytes>),
        {
            let name_spaces = self.issuer_signed.name_spaces.as_mut().unwrap();
            name_spaces.modify_attributes(name_space, modify_func);
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use futures::FutureExt;
    use itertools::Itertools;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::credential_payload::IntoCredentialPayload;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;

    use crate::holder::MdocCredentialPayloadError;
    use crate::holder::MdocParts;

    use super::Mdoc;

    #[test]
    fn test_from_mdoc() {
        let mdoc = Mdoc::new_mock().now_or_never().unwrap();
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::pid_example());

        let payload = mdoc
            .into_credential_payload(&metadata)
            .expect("creating and validating CredentialPayload from Mdoc should succeed");

        assert_eq!(
            payload.previewable_payload.attributes.into_values().collect_vec(),
            vec![
                Attribute::Single(AttributeValue::Text("De Bruijn".to_string())),
                Attribute::Single(AttributeValue::Text("Willeke Liselotte".to_string())),
                Attribute::Single(AttributeValue::Text("999999999".to_string()))
            ]
        );
    }

    #[test]
    fn test_from_mdoc_parts() {
        let mdoc = Mdoc::new_mock().now_or_never().unwrap();
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::pid_example());

        let payload = MdocParts::new(mdoc.issuer_signed.into_entries_by_namespace(), mdoc.mso)
            .into_credential_payload(&metadata)
            .expect("creating and validating CredentialPayload from Mdoc should succeed");

        assert_eq!(
            payload.previewable_payload.attributes.into_values().collect_vec(),
            vec![
                Attribute::Single(AttributeValue::Text("De Bruijn".to_string())),
                Attribute::Single(AttributeValue::Text("Willeke Liselotte".to_string())),
                Attribute::Single(AttributeValue::Text("999999999".to_string()))
            ]
        );
    }

    #[test]
    fn test_from_mdoc_parts_invalid() {
        let mdoc = Mdoc::new_mock().now_or_never().unwrap();
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_names(
            "urn:eudi:pid:nl:1",
            &[
                ("family_name", JsonSchemaPropertyType::Number, None),
                ("bsn", JsonSchemaPropertyType::String, None),
                ("given_name", JsonSchemaPropertyType::String, None),
            ],
        ));

        let error = MdocParts::new(mdoc.issuer_signed.into_entries_by_namespace(), mdoc.mso)
            .into_credential_payload(&metadata)
            .expect_err("wrong family_name type should fail validation");

        assert_matches!(error, MdocCredentialPayloadError::MetadataValidation(_));
    }
}
