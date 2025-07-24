use chrono::Utc;
use coset::CoseSign1;
use p256::ecdsa::VerifyingKey;
use ssri::Integrity;

use crypto::keys::EcdsaKey;
use crypto::server_keys::KeyPair;
use mdoc::DigestAlgorithm;
use mdoc::Error;
use mdoc::IssuerNameSpaces;
use mdoc::IssuerSigned;
use mdoc::MobileSecurityObject;
use mdoc::MobileSecurityObjectVersion;
use mdoc::Result;
use mdoc::ValidityInfo;
use mdoc::utils::cose::CoseKey;
use mdoc::utils::cose::MdocCose;
use mdoc::utils::serialization::TaggedBytes;

use crate::credential_payload::PreviewableCredentialPayload;

impl PreviewableCredentialPayload {
    pub async fn into_issuer_signed(
        self,
        metadata_integrity: Integrity,
        device_public_key: &VerifyingKey,
        key: &KeyPair<impl EcdsaKey>,
    ) -> Result<(IssuerSigned, MobileSecurityObject)> {
        let now = Utc::now();
        let validity = ValidityInfo {
            signed: now.into(),
            valid_from: self
                .not_before
                .map(Into::into)
                .ok_or_else(|| Error::MissingValidityInformation("valid_from".to_string()))?,
            valid_until: self
                .expires
                .map(Into::into)
                .ok_or_else(|| Error::MissingValidityInformation("valid_until".to_string()))?,
            expected_update: None,
        };

        let attributes = self.attributes.to_mdoc_attributes(&self.attestation_type);
        let attrs = IssuerNameSpaces::try_from(attributes).map_err(Error::MissingOrEmptyNamespace)?;

        let doc_type = self.attestation_type;
        let cose_pubkey: CoseKey = device_public_key.try_into()?;

        let mso = MobileSecurityObject {
            version: MobileSecurityObjectVersion::V1_0,
            digest_algorithm: DigestAlgorithm::SHA256,
            doc_type,
            value_digests: (&attrs).try_into()?,
            device_key_info: cose_pubkey.into(),
            validity_info: validity,
            issuer_uri: Some(self.issuer),
            attestation_qualification: Some(self.attestation_qualification),
            type_metadata_integrity: Some(metadata_integrity),
        };

        let header = IssuerSigned::create_unprotected_header(key.certificate().to_vec());

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
}

#[cfg(any(test, feature = "mock"))]
mod test {
    use p256::ecdsa::VerifyingKey;
    use ssri::Integrity;

    use crypto::CredentialEcdsaKey;
    use crypto::EcdsaKey;
    use crypto::server_keys::KeyPair;
    use mdoc::holder::Mdoc;

    use crate::credential_payload::PreviewableCredentialPayload;

    impl PreviewableCredentialPayload {
        /// Construct an [`Mdoc`] directly by signing, skipping validation.
        pub async fn into_signed_mdoc_unverified<K: CredentialEcdsaKey>(
            self,
            metadata_integrity: Integrity,
            private_key_id: String,
            public_key: &VerifyingKey,
            issuer_keypair: &KeyPair<impl EcdsaKey>,
        ) -> mdoc::Result<Mdoc> {
            let (issuer_signed, mso) = self
                .into_issuer_signed(metadata_integrity, public_key, issuer_keypair)
                .await?;

            let mdoc = Mdoc::new_unverified(mso, private_key_id, issuer_signed, K::KEY_TYPE);

            Ok(mdoc)
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use futures::FutureExt;
    use itertools::Itertools;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use ssri::Integrity;

    use crypto::EcdsaKey;
    use crypto::server_keys::generate::Ca;
    use mdoc::holder::Mdoc;
    use mdoc::test::generate_issuer_mock;
    use mdoc::utils::serialization::TaggedBytes;
    use mdoc::verifier::ValidityRequirement;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;
    use utils::generator::TimeGenerator;
    use utils::generator::mock::MockTimeGenerator;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;
    use crate::credential_payload::CredentialPayload;
    use crate::credential_payload::IntoCredentialPayload;
    use crate::credential_payload::MdocCredentialPayloadError;
    use crate::credential_payload::MdocParts;
    use crate::credential_payload::PreviewableCredentialPayload;

    #[tokio::test]
    async fn it_works() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_key = generate_issuer_mock(&ca).unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];

        let payload_preview: PreviewableCredentialPayload = CredentialPayload::example_with_attributes(
            vec![
                ("first_name", AttributeValue::Text("John".to_string())),
                ("family_name", AttributeValue::Text("Doe".to_string())),
            ],
            &issuance_key.verifying_key().await.unwrap(),
            &MockTimeGenerator::default(),
        )
        .previewable_payload;

        // Note that this resource integrity does not match any metadata source document.
        let metadata_integrity = Integrity::from(crypto::utils::random_bytes(32));
        let (issuer_signed, _) = payload_preview
            .clone()
            .into_issuer_signed(
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

    #[test]
    fn test_from_mdoc() {
        let mdoc = Mdoc::new_mock().now_or_never().unwrap();
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::pid_example());

        let payload = mdoc
            .into_credential_payload(&metadata)
            .expect("creating and validating CredentialPayload from Mdoc should succeed");

        assert_eq!(
            payload
                .previewable_payload
                .attributes
                .into_inner()
                .into_values()
                .collect_vec(),
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
            payload
                .previewable_payload
                .attributes
                .into_inner()
                .into_values()
                .collect_vec(),
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
