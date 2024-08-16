use chrono::Utc;
use ciborium::value::Value;
use coset::{CoseSign1, HeaderBuilder};

use crate::{
    iso::*,
    server_keys::KeyPair,
    unsigned::UnsignedMdoc,
    utils::{
        cose::{CoseKey, MdocCose, COSE_X5CHAIN_HEADER_LABEL},
        serialization::TaggedBytes,
    },
    Result,
};

impl IssuerSigned {
    // In the future, it should be an option to house the key used to sign IssuerSigned
    // within secure hardware. This is especially relevant for signing the PID.
    pub async fn sign(unsigned_mdoc: UnsignedMdoc, device_public_key: CoseKey, key: &KeyPair) -> Result<Self> {
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

        let headers = HeaderBuilder::new()
            .value(
                COSE_X5CHAIN_HEADER_LABEL,
                Value::Bytes(key.certificate().as_bytes().to_vec()),
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
    use std::{num::NonZeroU8, ops::Add};

    use ciborium::Value;
    use indexmap::IndexMap;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use wallet_common::{generator::TimeGenerator, keys::software::SoftwareEcdsaKey};

    use crate::{
        holder::Mdoc,
        server_keys::KeyPair,
        unsigned::{Entry, UnsignedMdoc},
        utils::{cose::CoseKey, issuer_auth::IssuerRegistration, serialization::TaggedBytes},
        verifier::ValidityRequirement,
        IssuerSigned,
    };

    const ISSUANCE_DOC_TYPE: &str = "example_doctype";
    const ISSUANCE_NAME_SPACE: &str = "example_namespace";
    const ISSUANCE_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

    #[tokio::test]
    async fn it_works() {
        let ca = KeyPair::generate_issuer_mock_ca().unwrap();
        let issuance_key = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();
        let trust_anchors = &[(ca.certificate()).try_into().unwrap()];

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

        let device_key = CoseKey::try_from(SigningKey::random(&mut OsRng).verifying_key()).unwrap();
        let issuer_signed = IssuerSigned::sign(unsigned.clone(), device_key, &issuance_key)
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
        let mdoc = Mdoc::new::<SoftwareEcdsaKey>(
            "key_id_not_used_in_this_test".to_string(),
            issuer_signed,
            &TimeGenerator,
            trust_anchors,
        )
        .unwrap();
        mdoc.compare_unsigned(&unsigned).unwrap();
    }
}
