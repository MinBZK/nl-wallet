use chrono::Utc;
use ciborium::value::Value;
use coset::CoseSign1;
use coset::Header;
use coset::HeaderBuilder;
use p256::ecdsa::VerifyingKey;
use ssri::Integrity;

use attestation_data::attributes::Attribute;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use crypto::keys::EcdsaKey;
use crypto::server_keys::KeyPair;

use crate::errors::Error;
use crate::iso::*;
use crate::utils::cose::CoseKey;
use crate::utils::cose::MdocCose;
use crate::utils::cose::COSE_X5CHAIN_HEADER_LABEL;
use crate::utils::serialization::TaggedBytes;
use crate::Result;

impl IssuerSigned {
    pub async fn sign(
        payload: PreviewableCredentialPayload,
        metadata_integrity: Integrity,
        device_public_key: &VerifyingKey,
        key: &KeyPair<impl EcdsaKey>,
    ) -> Result<(Self, MobileSecurityObject)> {
        let now = Utc::now();
        let validity = ValidityInfo {
            signed: now.into(),
            valid_from: payload
                .not_before
                .map(Into::into)
                .ok_or_else(|| Error::MissingValidityInformation("valid_from".to_string()))?,
            valid_until: payload
                .expires
                .map(Into::into)
                .ok_or_else(|| Error::MissingValidityInformation("valid_until".to_string()))?,
            expected_update: None,
        };

        let attributes = Attribute::from_attributes(&payload.attestation_type, payload.attributes);
        let attrs = IssuerNameSpaces::try_from(attributes).map_err(Error::MissingOrEmptyNamespace)?;

        let doc_type = payload.attestation_type;
        let cose_pubkey: CoseKey = device_public_key.try_into()?;

        let mso = MobileSecurityObject {
            version: MobileSecurityObjectVersion::V1_0,
            digest_algorithm: DigestAlgorithm::SHA256,
            doc_type,
            value_digests: (&attrs).try_into()?,
            device_key_info: cose_pubkey.into(),
            validity_info: validity,
            issuer_uri: Some(payload.issuer),
            attestation_qualification: Some(payload.attestation_qualification),
            type_metadata_integrity: Some(metadata_integrity),
        };

        let header = Self::create_unprotected_header(key.certificate().to_vec());

        let mso_tagged = TaggedBytes(mso);
        let issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            MdocCose::sign(&mso_tagged, header, key, true).await?;
        let TaggedBytes(mso) = mso_tagged;

        let issuer_signed = IssuerSigned {
            name_spaces: attrs.into(),
            issuer_auth,
        };

        Ok((issuer_signed, mso))
    }

    pub(crate) fn create_unprotected_header(x5chain: Vec<u8>) -> Header {
        HeaderBuilder::new()
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(x5chain))
            .build()
    }

    #[cfg(any(test, feature = "test"))]
    pub async fn resign(&mut self, key: &KeyPair<impl EcdsaKey>) -> Result<()> {
        let mut mso = self.issuer_auth.dangerous_parse_unverified()?.0;

        // Update (fill) the issuer_uri to match the new key
        mso.issuer_uri = Some(key.certificate().san_dns_name_or_uris()?.into_first());

        self.issuer_auth = MdocCose::sign(&mso.into(), self.issuer_auth.0.unprotected.clone(), key, true).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use attestation_data::attributes::AttributeValue;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::credential_payload::CredentialPayload;
    use attestation_data::credential_payload::PreviewableCredentialPayload;
    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use crypto::server_keys::generate::Ca;
    use crypto::EcdsaKey;
    use ssri::Integrity;
    use utils::generator::TimeGenerator;

    use crate::utils::serialization::TaggedBytes;
    use crate::verifier::ValidityRequirement;
    use crate::IssuerSigned;

    #[tokio::test]
    async fn it_works() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_key = generate_issuer_mock(&ca, IssuerRegistration::new_mock().into()).unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];

        let payload_preview: PreviewableCredentialPayload = CredentialPayload::example_with_attributes(
            vec![
                ("first_name", AttributeValue::Text("John".to_string())),
                ("family_name", AttributeValue::Text("Doe".to_string())),
            ],
            &issuance_key.verifying_key().await.unwrap(),
        )
        .previewable_payload;

        // Note that this resource integrity does not match any metadata source document.
        let metadata_integrity = Integrity::from(crypto::utils::random_bytes(32));
        let (issuer_signed, _) = IssuerSigned::sign(
            payload_preview.clone(),
            metadata_integrity.clone(),
            SigningKey::random(&mut OsRng).verifying_key(),
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
        assert_eq!(cose_payload.doc_type, payload_preview.attestation_type);
        assert_eq!(
            payload_preview.not_before.unwrap(),
            (&cose_payload.validity_info.valid_from).try_into().unwrap(),
        );
        assert_eq!(
            payload_preview.expires.unwrap(),
            (&cose_payload.validity_info.valid_until).try_into().unwrap(),
        );
        assert_eq!(cose_payload.type_metadata_integrity, Some(metadata_integrity));
    }
}
