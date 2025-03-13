use indexmap::IndexMap;

use mdoc::unsigned::Entry;
use mdoc::utils::auth::Organization;
use mdoc::NameSpace;
use openid4vc::attributes::Attribute;
use sd_jwt::metadata::TypeMetadata;

use super::Attestation;
use super::AttestationError;
use super::AttestationIdentity;
use super::AttributeSelectionMode;

impl Attestation {
    pub(crate) fn create_for_issuance(
        identity: AttestationIdentity,
        attestation_type: String,
        metadata: TypeMetadata,
        issuer_organization: Organization,
        mdoc_attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, AttestationError> {
        let nested_attributes = Attribute::from_mdoc_attributes(&attestation_type, mdoc_attributes)?;

        Self::create_from_attributes(
            identity,
            attestation_type,
            metadata,
            issuer_organization,
            nested_attributes,
            AttributeSelectionMode::Issuance,
        )
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use indexmap::IndexMap;

    use mdoc::unsigned::Entry;
    use mdoc::utils::auth::Organization;
    use mdoc::DataElementValue;
    use openid4vc::attributes::AttributeValue;
    use sd_jwt::metadata::TypeMetadata;

    use crate::attestation::attribute::test::claim_metadata;
    use crate::attestation::AttestationError;
    use crate::Attestation;
    use crate::AttestationIdentity;

    fn example_mdoc_attributes() -> IndexMap<String, Vec<Entry>> {
        IndexMap::from([(
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
        )])
    }

    #[test]
    fn test_happy() {
        let metadata = TypeMetadata {
            claims: vec![
                claim_metadata(&["namespace1", "entry1"]),
                claim_metadata(&["namespace1", "entry2"]),
            ],
            ..TypeMetadata::empty_example()
        };

        let attestation = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            String::from("example_attestation_type"),
            metadata,
            Organization::new_mock(),
            example_mdoc_attributes(),
        )
        .expect("creating new Attestation should be successful");

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
    fn test_attribute_not_found() {
        let metadata = TypeMetadata {
            claims: vec![claim_metadata(&["not_found"])],
            ..TypeMetadata::empty_example()
        };

        let error = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            String::from("example_attestation_type"),
            metadata,
            Organization::new_mock(),
            example_mdoc_attributes(),
        )
        .expect_err("creating new Attestation should not be successful");

        assert_matches!(error, AttestationError::AttributeNotFoundForClaim(_));
    }

    #[test]
    fn test_attribute_not_processed() {
        let metadata = TypeMetadata {
            claims: vec![claim_metadata(&["namespace1", "entry1"])],
            ..TypeMetadata::empty_example()
        };

        let error = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            String::from("example_attestation_type"),
            metadata,
            Organization::new_mock(),
            example_mdoc_attributes(),
        )
        .expect_err("creating new Attestation should not be successful");

        assert_matches!(
            error,
            AttestationError::AttributeNotProcessedByClaim(keys)
                if keys == HashSet::from([vec![String::from("namespace1"), String::from("entry2")]])
        );
    }
}
