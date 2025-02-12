use nl_wallet_mdoc::utils::auth::Organization;
use openid4vc::credential_payload::CredentialPayload;
use sd_jwt::metadata::TypeMetadata;

use crate::attestation::AttestationError;
use crate::attestation::AttestationIdentity;
use crate::attestation::AttributeSelectionMode;
use crate::Attestation;
use crate::AttestationAttribute;

impl Attestation {
    pub(crate) fn create_for_issuance(
        identity: AttestationIdentity,
        payload: CredentialPayload,
        metadata: TypeMetadata,
        issuer_organization: Organization,
    ) -> Result<Self, AttestationError> {
        let attributes =
            AttestationAttribute::from_attributes(&payload.attributes, &metadata, &AttributeSelectionMode::Issuance)?;

        Self::create_from_attributes(
            identity,
            payload.attestation_type,
            metadata.display,
            issuer_organization,
            attributes,
            &payload.attributes,
        )
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use chrono::Utc;
    use http::Uri;

    use nl_wallet_mdoc::utils::auth::Organization;
    use openid4vc::attributes::AttributeValue;
    use openid4vc::credential_payload::CredentialPayload;
    use sd_jwt::metadata::TypeMetadata;

    use crate::attestation::attribute::test::claim_metadata;
    use crate::attestation::attribute::test::ATTRIBUTES;
    use crate::attestation::AttestationError;
    use crate::Attestation;
    use crate::AttestationIdentity;

    fn example_credential_payload() -> CredentialPayload {
        let attributes = &*ATTRIBUTES;

        CredentialPayload {
            attestation_type: String::from("pid123"),
            issuer: Uri::from_static("data://org.example.com/org2"),
            issued_at: Some(Utc::now()),
            expires: Some(Utc::now()),
            not_before: None,
            attributes: attributes.clone(),
        }
    }

    #[test]
    fn test_happy() {
        let mut metadata = TypeMetadata::bsn_only_example();
        metadata.claims = vec![
            claim_metadata(&["single"]),
            claim_metadata(&["nested_1a", "nested_1b", "nested_1c"]),
        ];

        let payload = example_credential_payload();

        let attestation = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            payload,
            metadata,
            Organization::new_mock(),
        )
        .unwrap();

        let attrs = attestation
            .attributes
            .iter()
            .map(|attr| (attr.key.clone(), attr.value.clone()))
            .collect::<Vec<_>>();

        assert_eq!(
            [
                (
                    vec![String::from("single")],
                    AttributeValue::Text(String::from("single"))
                ),
                (
                    vec!["nested_1a", "nested_1b", "nested_1c"]
                        .into_iter()
                        .map(String::from)
                        .collect(),
                    AttributeValue::Text(String::from("nested_value")),
                )
            ],
            attrs.as_slice()
        );
    }

    #[test]
    fn test_attribute_not_found() {
        let mut metadata = TypeMetadata::bsn_only_example();
        metadata.claims = vec![claim_metadata(&["not_found"])];

        let payload = example_credential_payload();

        let attestation = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            payload,
            metadata,
            Organization::new_mock(),
        );
        assert_matches!(attestation, Err(AttestationError::AttributeNotFoundForClaim(_)));
    }

    #[test]
    fn test_attribute_not_processed() {
        let mut metadata = TypeMetadata::bsn_only_example();
        metadata.claims = vec![claim_metadata(&["nested_1a", "nested_1b", "nested_1c"])];

        let payload = example_credential_payload();

        let attestation = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            payload,
            metadata,
            Organization::new_mock(),
        );
        assert_matches!(
            attestation,
            Err(AttestationError::AttributeNotProcessedByClaim(keys)) if keys == vec![vec![String::from("single")]]
        );
    }
}
