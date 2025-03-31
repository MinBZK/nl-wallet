use chrono::Utc;
use ciborium::value::Value;
use coset::CoseSign1;
use coset::Header;
use coset::HeaderBuilder;
use coset::Label;
use ssri::Integrity;

use crypto::keys::EcdsaKey;
use crypto::server_keys::KeyPair;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::SD_JWT_VC_TYPE_METADATA_KEY;

use crate::iso::*;
use crate::unsigned::UnsignedMdoc;
use crate::utils::cose::CoseKey;
use crate::utils::cose::MdocCose;
use crate::utils::cose::COSE_X5CHAIN_HEADER_LABEL;
use crate::utils::serialization::CborError;
use crate::utils::serialization::TaggedBytes;
use crate::Result;

impl IssuerSigned {
    pub async fn sign(
        unsigned_mdoc: UnsignedMdoc,
        metadata_integrity: Integrity,
        metadata_documents: &TypeMetadataDocuments,
        device_public_key: CoseKey,
        key: &KeyPair<impl EcdsaKey>,
    ) -> Result<Self> {
        let now = Utc::now();
        let validity = ValidityInfo {
            signed: now.into(),
            valid_from: unsigned_mdoc.valid_from,
            valid_until: unsigned_mdoc.valid_until,
            expected_update: None,
        };

        let doc_type = unsigned_mdoc.doc_type;
        let attrs = IssuerNameSpaces::from(unsigned_mdoc.attributes);

        let mso = MobileSecurityObject {
            version: MobileSecurityObjectVersion::V1_0,
            digest_algorithm: DigestAlgorithm::SHA256,
            doc_type,
            value_digests: (&attrs).try_into()?,
            device_key_info: device_public_key.into(),
            validity_info: validity,
            issuer_uri: Some(unsigned_mdoc.issuer_uri),
            attestation_qualification: Some(unsigned_mdoc.attestation_qualification),
            type_metadata_integrity: Some(metadata_integrity),
        };

        let headers = Self::create_unprotected_header(key.certificate().to_vec(), metadata_documents)?;

        let mso_tagged = mso.into();
        let issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            MdocCose::sign(&mso_tagged, headers, key, true).await?;

        let issuer_signed = IssuerSigned {
            name_spaces: attrs.into(),
            issuer_auth,
        };

        Ok(issuer_signed)
    }

    pub(crate) fn create_unprotected_header(
        x5chain: Vec<u8>,
        metadata_documents: &TypeMetadataDocuments,
    ) -> Result<Header> {
        let header = HeaderBuilder::new()
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(x5chain))
            .text_value(
                String::from(SD_JWT_VC_TYPE_METADATA_KEY),
                Value::serialized(metadata_documents).map_err(CborError::Value)?,
            )
            .build();

        Ok(header)
    }

    pub fn type_metadata_documents(&self) -> Result<TypeMetadataDocuments> {
        let documents_label = Label::Text(String::from(SD_JWT_VC_TYPE_METADATA_KEY));

        let metadata_documents = self
            .issuer_auth
            .unprotected_header_item(&documents_label)?
            .deserialized()
            .map_err(CborError::Value)?;

        Ok(metadata_documents)
    }

    #[cfg(any(test, feature = "test"))]
    pub async fn resign(&mut self, key: &KeyPair<impl EcdsaKey>, metadata_integrity: Integrity) -> Result<()> {
        let mut mso = self.issuer_auth.dangerous_parse_unverified()?.0;

        // Update (fill) the issuer_uri to match the new key
        mso.issuer_uri = Some(key.certificate().san_dns_name_or_uris()?.into_first());

        mso.type_metadata_integrity = Some(metadata_integrity);

        self.issuer_auth = MdocCose::sign(&mso.into(), self.issuer_auth.0.unprotected.clone(), key, true).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;
    use std::ops::Add;

    use chrono::Days;
    use ciborium::Value;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use indexmap::IndexMap;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use wallet_common::generator::TimeGenerator;

    use crate::holder::Mdoc;
    use crate::server_keys::generate::mock::generate_issuer_mock;
    use crate::unsigned::Entry;
    use crate::unsigned::UnsignedMdoc;
    use crate::utils::cose::CoseKey;
    use crate::utils::issuer_auth::IssuerRegistration;
    use crate::utils::serialization::TaggedBytes;
    use crate::verifier::ValidityRequirement;
    use crate::IssuerSigned;

    const ISSUANCE_DOC_TYPE: &str = "example_doctype";
    const ISSUANCE_NAME_SPACE: &str = "example_namespace";
    const ISSUANCE_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

    #[tokio::test]
    async fn it_works() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_key = generate_issuer_mock(&ca, IssuerRegistration::new_mock().into()).unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];

        let now = chrono::Utc::now();
        let unsigned = UnsignedMdoc {
            doc_type: ISSUANCE_DOC_TYPE.to_string(),
            copy_count: NonZeroU8::new(2).unwrap(),
            valid_from: now.into(),
            valid_until: now.add(Days::new(365)).into(),
            attributes: IndexMap::from([(
                ISSUANCE_NAME_SPACE.to_string(),
                ISSUANCE_ATTRS
                    .iter()
                    .map(|(key, val)| Entry {
                        name: key.to_string(),
                        value: Value::Text(val.to_string()),
                    })
                    .collect(),
            )])
            .try_into()
            .unwrap(),
            issuer_uri: issuance_key.certificate().san_dns_name_or_uris().unwrap().into_first(),
            attestation_qualification: Default::default(),
        };

        // NOTE: This metadata does not match the attributes.
        let (_, metadata_integrity, metadata_documents) = TypeMetadataDocuments::from_single_example(
            TypeMetadata::empty_example_with_attestation_type(ISSUANCE_DOC_TYPE),
        );
        let device_key = CoseKey::try_from(SigningKey::random(&mut OsRng).verifying_key()).unwrap();
        let issuer_signed = IssuerSigned::sign(
            unsigned.clone(),
            metadata_integrity.clone(),
            &metadata_documents,
            device_key,
            &issuance_key,
        )
        .await
        .unwrap();

        // The IssuerSigned should be valid
        issuer_signed
            .verify(ValidityRequirement::Valid, &TimeGenerator, trust_anchors)
            .unwrap();

        // The issuer certificate generated above should be included in the IssuerAuth
        assert_eq!(
            &issuer_signed.issuer_auth.signing_cert().unwrap(),
            issuance_key.certificate()
        );

        let TaggedBytes(cose_payload) = issuer_signed.issuer_auth.dangerous_parse_unverified().unwrap();
        assert_eq!(cose_payload.doc_type, unsigned.doc_type);
        assert_eq!(cose_payload.validity_info.valid_from, unsigned.valid_from);
        assert_eq!(cose_payload.validity_info.valid_until, unsigned.valid_until);
        assert_eq!(cose_payload.type_metadata_integrity, Some(metadata_integrity));

        let received_metadata_documents = issuer_signed
            .type_metadata_documents()
            .expect("retrieving type metadata documents from IssuerSigned should succeed");

        assert_eq!(received_metadata_documents, metadata_documents);

        // Construct an mdoc so we can use `compare_unsigned()` to check that the attributes have the expected values
        let mdoc = Mdoc::new::<MockRemoteEcdsaKey>(
            "key_id_not_used_in_this_test".to_string(),
            issuer_signed,
            &TimeGenerator,
            trust_anchors,
        )
        .unwrap();
        mdoc.compare_unsigned(&unsigned).unwrap();
    }
}
