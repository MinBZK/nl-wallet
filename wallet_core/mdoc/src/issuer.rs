use chrono::Utc;
use ciborium::value::Value;
use coset::CoseSign1;
use coset::HeaderBuilder;

use sd_jwt::metadata::TypeMetadataChain;
use sd_jwt::metadata::TypeMetadataError;
use sd_jwt::metadata::COSE_METADATA_HEADER_LABEL;
use sd_jwt::metadata::COSE_METADATA_INTEGRITY_HEADER_LABEL;
use wallet_common::keys::EcdsaKey;

use crate::iso::*;
use crate::server_keys::KeyPair;
use crate::unsigned::UnsignedMdoc;
use crate::utils::cose::CoseKey;
use crate::utils::cose::MdocCose;
use crate::utils::cose::COSE_X5CHAIN_HEADER_LABEL;
use crate::utils::serialization::TaggedBytes;
use crate::Result;

impl IssuerSigned {
    pub async fn sign(
        unsigned_mdoc: UnsignedMdoc,
        metadata_chain: TypeMetadataChain,
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
        };

        // TODO: verify JSON representation of unsigned_mdoc against metadata schema (PVW-3808)

        let (chain, integrity) = metadata_chain.into_destructured();

        let headers = HeaderBuilder::new()
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key.certificate().to_vec()))
            .text_value(
                String::from(COSE_METADATA_HEADER_LABEL),
                Value::Array(
                    chain
                        .into_iter()
                        .map(|m| {
                            let encoded = m.try_as_base64()?;
                            Ok(Value::Text(encoded))
                        })
                        .collect::<Result<_, TypeMetadataError>>()?,
                ),
            )
            .text_value(
                String::from(COSE_METADATA_INTEGRITY_HEADER_LABEL),
                Value::Text(integrity.into()),
            )
            .build();
        let mso_tagged = mso.into();
        let issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            MdocCose::sign(&mso_tagged, headers, key, true).await?;

        let issuer_signed = IssuerSigned {
            name_spaces: attrs.into(),
            issuer_auth,
        };

        Ok(issuer_signed)
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;
    use std::ops::Add;

    use ciborium::Value;
    use indexmap::IndexMap;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use sd_jwt::metadata::TypeMetadata;
    use sd_jwt::metadata::TypeMetadataChain;
    use wallet_common::generator::TimeGenerator;
    use wallet_common::keys::mock_remote::MockRemoteEcdsaKey;

    use crate::holder::Mdoc;
    use crate::server_keys::generate::Ca;
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
        let issuance_key = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];

        let unsigned = UnsignedMdoc {
            doc_type: ISSUANCE_DOC_TYPE.to_string(),
            copy_count: NonZeroU8::new(2).unwrap(),
            valid_from: chrono::Utc::now().into(),
            valid_until: chrono::Utc::now().add(chrono::Duration::days(365)).into(),
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
        };
        let metadata = TypeMetadata::new_example();
        let metadata_chain = TypeMetadataChain::create(metadata, vec![]).unwrap();

        let device_key = CoseKey::try_from(SigningKey::random(&mut OsRng).verifying_key()).unwrap();
        let issuer_signed = IssuerSigned::sign(unsigned.clone(), metadata_chain, device_key, &issuance_key)
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
