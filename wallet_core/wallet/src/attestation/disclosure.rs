use indexmap::IndexMap;

use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::utils::auth::Organization;
use nl_wallet_mdoc::NameSpace;
use openid4vc::attributes::Attribute;
use sd_jwt::metadata::TypeMetadata;

use crate::attestation::AttestationError;
use crate::attestation::AttributeSelectionMode;
use crate::Attestation;
use crate::AttestationAttribute;
use crate::AttestationIdentity;

impl Attestation {
    pub(crate) fn create_for_disclosure(
        identity: AttestationIdentity,
        attestation_type: String,
        mdoc_attributes: &IndexMap<NameSpace, Vec<Entry>>,
        metadata: TypeMetadata,
        issuer_organization: Organization,
    ) -> Result<Self, AttestationError> {
        let attributes_by_key = Attribute::from_mdoc_attributes(&attestation_type, mdoc_attributes)?;
        let attributes =
            AttestationAttribute::from_attributes(&attributes_by_key, &metadata, &AttributeSelectionMode::Disclosure)?;

        Self::create_from_attributes(
            identity,
            attestation_type,
            metadata.display,
            issuer_organization,
            attributes,
            &attributes_by_key,
        )
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use nl_wallet_mdoc::unsigned::Entry;
    use nl_wallet_mdoc::utils::auth::Organization;
    use nl_wallet_mdoc::DataElementValue;
    use openid4vc::attributes::AttributeValue;
    use sd_jwt::metadata::TypeMetadata;

    use crate::attestation::attribute::test::claim_metadata;
    use crate::attestation::AttestationError;
    use crate::Attestation;
    use crate::AttestationIdentity;

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
            AttestationIdentity::Ephemeral,
            String::from("example_attestation_type"),
            &mdoc_attributes,
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
            AttestationIdentity::Ephemeral,
            String::from("example_attestation_type"),
            &mdoc_attributes,
            metadata,
            Organization::new_mock(),
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
            AttestationIdentity::Ephemeral,
            String::from("example_attestation_type"),
            &mdoc_attributes,
            metadata,
            Organization::new_mock(),
        );

        assert_matches!(
            attestation,
            Err(AttestationError::AttributeNotProcessedByClaim(claims)) if claims == vec![vec!["namespace1", "entry2"]]
        );
    }
}
