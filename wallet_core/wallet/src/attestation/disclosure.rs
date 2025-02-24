use indexmap::IndexMap;

use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::utils::auth::Organization;
use nl_wallet_mdoc::NameSpace;
use openid4vc::attributes::Attribute;
use sd_jwt::metadata::TypeMetadata;

use super::Attestation;
use super::AttestationError;
use super::AttestationIdentity;
use super::AttributeSelectionMode;

impl Attestation {
    pub(crate) fn create_for_disclosure(
        attestation_type: String,
        metadata: TypeMetadata,
        issuer_organization: Organization,
        mdoc_attributes: &IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, AttestationError> {
        let nested_attributes = Attribute::from_mdoc_attributes(&attestation_type, mdoc_attributes)?;

        Self::create_from_attributes(
            AttestationIdentity::Ephemeral,
            attestation_type,
            metadata,
            issuer_organization,
            nested_attributes,
            AttributeSelectionMode::Disclosure,
        )
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use indexmap::IndexMap;

    use nl_wallet_mdoc::unsigned::Entry;
    use nl_wallet_mdoc::utils::auth::Organization;
    use nl_wallet_mdoc::DataElementValue;
    use openid4vc::attributes::AttributeValue;
    use sd_jwt::metadata::TypeMetadata;

    use crate::attestation::attribute::test::claim_metadata;
    use crate::attestation::Attestation;
    use crate::attestation::AttestationError;

    #[test]
    fn test_happy() {
        let mut metadata = TypeMetadata::bsn_only_example();
        metadata.claims = vec![
            claim_metadata(&["namespace1", "entry1"]),
            claim_metadata(&["namespace1", "entry2"]),
        ];

        let mdoc_attributes = IndexMap::from([(
            String::from("example_attestation_type.namespace1"),
            vec![
                Entry {
                    name: String::from("entry1"),
                    value: DataElementValue::Text(String::from("value1")),
                },
                Entry {
                    name: String::from("entry2"),
                    value: DataElementValue::Bool(true),
                },
            ],
        )]);

        let attestation = Attestation::create_for_disclosure(
            String::from("example_attestation_type"),
            metadata,
            Organization::new_mock(),
            &mdoc_attributes,
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
                    vec![String::from("namespace1"), String::from("entry1")],
                    AttributeValue::Text(String::from("value1"))
                ),
                (
                    vec![String::from("namespace1"), String::from("entry2")],
                    AttributeValue::Bool(true)
                ),
            ],
            attrs.as_slice()
        );
    }

    #[test]
    fn test_not_all_claims_need_to_match_attributes() {
        let mut metadata = TypeMetadata::bsn_only_example();
        metadata.claims = vec![
            claim_metadata(&["namespace1", "entry1"]),
            claim_metadata(&["namespace1", "entry2"]),
        ];

        let mdoc_attributes = IndexMap::from([(
            String::from("example_attestation_type.namespace1"),
            vec![Entry {
                name: String::from("entry1"),
                value: DataElementValue::Text(String::from("value1")),
            }],
        )]);

        let attestation = Attestation::create_for_disclosure(
            String::from("example_attestation_type"),
            metadata,
            Organization::new_mock(),
            &mdoc_attributes,
        );

        assert!(attestation.is_ok());
    }

    #[test]
    fn test_attributes_not_processed() {
        let mut metadata = TypeMetadata::bsn_only_example();
        metadata.claims = vec![claim_metadata(&["namespace1", "entry1"])];

        let mdoc_attributes = IndexMap::from([(
            String::from("example_attestation_type.namespace1"),
            vec![
                Entry {
                    name: String::from("entry1"),
                    value: DataElementValue::Text(String::from("value1")),
                },
                Entry {
                    name: String::from("entry2"),
                    value: DataElementValue::Text(String::from("value2")),
                },
            ],
        )]);

        let attestation = Attestation::create_for_disclosure(
            String::from("example_attestation_type"),
            metadata,
            Organization::new_mock(),
            &mdoc_attributes,
        );

        assert_matches!(
            attestation,
            Err(AttestationError::AttributeNotProcessedByClaim(claims))
                if claims == HashSet::from([vec![String::from("namespace1"), String::from("entry2")]])
        );
    }
}
