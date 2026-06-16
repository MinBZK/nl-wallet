use std::collections::HashSet;

use attestation_data::attributes::Attributes;
use attestation_data::auth::Organization;
use attestation_types::claim_path::ClaimPath;
use attestation_types::credential_format::Format;
use indexmap::IndexMap;
use mdoc::iso::mdocs::Entry;
use mdoc::iso::mdocs::NameSpace;
use sd_jwt::claims::ObjectClaims;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use super::AttestationAttribute;
use super::AttestationError;
use super::AttestationIdentity;
use super::AttestationPresentation;
use super::AttestationPresentationConfig;
use super::AttestationValidity;

impl AttestationPresentation {
    pub(crate) fn create_from_mdoc(
        identity: AttestationIdentity,
        metadata: NormalizedTypeMetadata,
        issuer_organization: Box<Organization>,
        validity: AttestationValidity,
        mdoc_attributes: IndexMap<NameSpace, Vec<Entry>>,
        config: &impl AttestationPresentationConfig,
    ) -> Result<Self, AttestationError> {
        let nested_attributes = Attributes::from_mdoc_attributes(&metadata, mdoc_attributes)?;

        Self::create_from_attributes(
            identity,
            Format::MsoMdoc,
            metadata,
            issuer_organization,
            validity,
            &nested_attributes,
            config,
        )
    }

    pub(crate) fn create_from_sd_jwt_claims(
        identity: AttestationIdentity,
        metadata: NormalizedTypeMetadata,
        issuer_organization: Box<Organization>,
        validity: AttestationValidity,
        sd_jwt_claims: ObjectClaims,
        config: &impl AttestationPresentationConfig,
    ) -> Result<Self, AttestationError> {
        let attributes: Attributes = sd_jwt_claims.try_into()?;

        Self::create_from_attributes(
            identity,
            Format::SdJwt,
            metadata,
            issuer_organization,
            validity,
            &attributes,
            config,
        )
    }

    // Construct a new `AttestationPresentation` from a combination of metadata and nested attributes.
    #[expect(clippy::too_many_arguments, reason = "internal constructor")]
    pub(crate) fn create_from_attributes(
        identity: AttestationIdentity,
        format: Format,
        metadata: NormalizedTypeMetadata,
        issuer: Box<Organization>,
        validity: AttestationValidity,
        nested_attributes: &Attributes,
        config: &impl AttestationPresentationConfig,
    ) -> Result<Self, AttestationError> {
        let (attestation_type, display_metadata, claims) = metadata.into_presentation_components();

        // For every claim in the metadata, find the correct attribute
        // and convert it to a `AttestationAttribute` value (with optionally Json Schema metadata).
        let flattened_attributes = nested_attributes.flattened();
        let mut attributes = Vec::with_capacity(flattened_attributes.len());
        for claim in claims {
            let Some(claim_path) = claim
                .path
                .into_iter()
                .map(ClaimPath::try_into_key_path)
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            // This is safe as `claim.path` is non-empty.
            let claim_path = VecNonEmpty::try_from(claim_path).unwrap();

            // Get value of claim out of the nested attributes via flattened view
            // Cannot use swap_remove here to make the error checking easier
            let path_with_refs = claim_path
                .nonempty_iter()
                .map(String::as_str)
                .collect::<VecNonEmpty<_>>();
            if let Some(&value) = flattened_attributes.get(&path_with_refs) {
                attributes.push(AttestationAttribute {
                    key: claim_path,
                    metadata: claim.display,
                    value: value.to_owned(),
                    svg_id: claim.svg_id.map(String::from),
                })
            }
        }

        // Should not happen as the attributes should be validated by `Attributes::validate`
        // on receiving which does same check
        if attributes.len() != flattened_attributes.len() {
            let mut paths = flattened_attributes
                .into_iter()
                .map(|(path, _)| path.iter().map(ToString::to_string).collect::<Vec<_>>())
                .collect::<HashSet<_>>();
            for attribute in attributes {
                paths.remove(attribute.key.as_ref());
            }
            return Err(AttestationError::AttributesNotProcessedByClaim(paths));
        }

        let attributes = match config.filtered_attribute(&attestation_type) {
            Some(filtered_key) => attributes
                .into_iter()
                .filter(|attr| attr.key.iter().ne(filtered_key))
                .collect(),
            None => attributes,
        };

        // Finally, construct the `AttestationPresentation` type.
        Ok(AttestationPresentation {
            identity,
            format,
            display_metadata,
            attestation_type,
            issuer,
            attributes,
            validity,
        })
    }
}

#[cfg(test)]
pub mod test {
    use std::assert_matches;
    use std::collections::HashSet;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use attestation_data::auth::Organization;
    use attestation_data::validity::ValidityWindow;
    use attestation_types::claim_path::ClaimPath;
    use attestation_types::credential_format::Format;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_BSN;
    use attestation_types::pid_constants::PID_RECOVERY_CODE;
    use indexmap::IndexMap;
    use mdoc::iso::mdocs::DataElementValue;
    use mdoc::iso::mdocs::Entry;
    use sd_jwt_vc_metadata::ClaimDisplayMetadata;
    use sd_jwt_vc_metadata::ClaimMetadata;
    use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;
    use serde_json::json;
    use utils::vec_nonempty;

    use super::super::AttestationAttribute;
    use super::super::AttestationError;
    use super::super::AttestationIdentity;
    use super::super::AttestationPresentation;
    use super::super::AttributesError;
    use super::super::mock::EmptyPresentationConfig;
    use crate::attestation::AttestationValidity;
    use crate::config::test::test_wallet_config;

    fn claim_metadata(keys: &[&str]) -> ClaimMetadata {
        ClaimMetadata {
            path: keys
                .iter()
                .map(|key| ClaimPath::SelectByKey(String::from(*key)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            display: vec![],
            sd: ClaimSelectiveDisclosureMetadata::Always,
            mandatory: false,
            svg_id: None,
        }
    }

    fn example_metadata() -> NormalizedTypeMetadata {
        NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata {
            vct: String::from("example_attestation_type"),
            claims: vec![claim_metadata(&["entry1"]), claim_metadata(&["entry2"])],
            ..UncheckedTypeMetadata::example_with_claim_names("example_attestation_type", &["entry1", "entry2"])
        })
    }

    #[test]
    fn test_create_from_mdoc() {
        let mdoc_attributes = IndexMap::from([(
            String::from("example_attestation_type"),
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

        let attestation = AttestationPresentation::create_from_mdoc(
            AttestationIdentity::Ephemeral,
            example_metadata(),
            Organization::new_mock(),
            AttestationValidity {
                revocation_status: None,
                validity_window: ValidityWindow::new_valid_mock(),
            },
            mdoc_attributes,
            &EmptyPresentationConfig,
        )
        .expect("creating AttestationPresentation should succeed");

        let attrs = attestation
            .attributes
            .iter()
            .map(|attr| (attr.key.clone(), attr.value.clone()))
            .collect::<Vec<_>>();

        assert_eq!(
            [
                (
                    vec_nonempty![String::from("entry1")],
                    AttributeValue::Text(String::from("value1"))
                ),
                (vec_nonempty![String::from("entry2")], AttributeValue::Bool(true)),
            ],
            attrs.as_slice()
        );
    }

    #[test]
    fn test_create_from_mdoc_partial() {
        let mdoc_attributes = IndexMap::from([(
            String::from("example_attestation_type"),
            vec![Entry {
                name: String::from("entry1"),
                value: DataElementValue::Text(String::from("value1")),
            }],
        )]);

        let attestation = AttestationPresentation::create_from_mdoc(
            AttestationIdentity::Ephemeral,
            example_metadata(),
            Organization::new_mock(),
            AttestationValidity {
                revocation_status: None,
                validity_window: ValidityWindow::new_valid_mock(),
            },
            mdoc_attributes,
            &EmptyPresentationConfig,
        )
        .expect("creating AttestationPresentation should succeed");

        assert_eq!(attestation.attributes.len(), 1);
    }

    #[test]
    fn test_create_from_mdoc_error_some_attributes_not_processed() {
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata {
            vct: String::from("example_attestation_type"),
            claims: vec![claim_metadata(&["entry1"])],
            ..UncheckedTypeMetadata::example_with_claim_names("example_attestation_type", &["entry1"])
        });

        let mdoc_attributes = IndexMap::from([(
            String::from("example_attestation_type"),
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

        let error = AttestationPresentation::create_from_mdoc(
            AttestationIdentity::Ephemeral,
            metadata,
            Organization::new_mock(),
            AttestationValidity {
                revocation_status: None,
                validity_window: ValidityWindow::new_valid_mock(),
            },
            mdoc_attributes,
            &EmptyPresentationConfig,
        )
        .expect_err("creating AttestationPresentation should not succeed");

        assert_matches!(
            error,
            AttestationError::Attributes(AttributesError::SomeAttributesNotProcessed(claims))
                if *claims == IndexMap::from([
                    (String::from("example_attestation_type"),
                    vec![Entry {
                        name: String::from("entry2"),
                        value: ciborium::value::Value::Text(String::from("value2"))
                    }]
                )]
            )
        );
    }

    fn example_attributes() -> Attributes {
        IndexMap::from([
            (
                "name".to_string(),
                Attribute::Single(AttributeValue::Text("Wallet".to_string())),
            ),
            (
                "birth_date".to_string(),
                Attribute::Single(AttributeValue::Text("1996-06-16".to_string())),
            ),
            (
                "address".to_string(),
                Attribute::Nested(IndexMap::from([
                    (
                        "street".to_string(),
                        Attribute::Single(AttributeValue::Text("Gracht".to_string())),
                    ),
                    ("number".to_string(), Attribute::Single(AttributeValue::Integer(123))),
                ])),
            ),
        ])
        .into()
    }

    #[test]
    fn test_create_from_attributes() {
        let attributes = example_attributes();
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"locale": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["name"],
                    "display": [{"locale": "en", "label": "name"}],
                },
                {
                    "path": ["birth_date"],
                    "display": [{"locale": "en", "label": "birth date"}],
                },
                {
                    "path": ["address", "street"],
                    "display": [{"locale": "en", "label": "address street"}],
                },
                {
                    "path": ["address", "number"],
                    "display": [{"locale": "en", "label": "address number"}],
                },
                {
                    "path": ["country", "iso"],
                    "display": [{"locale": "en", "label": "country iso"}],
                },
            ]
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let attestation_presentation = AttestationPresentation::create_from_attributes(
            AttestationIdentity::Ephemeral,
            Format::SdJwt,
            type_metadata,
            Organization::new_mock(),
            AttestationValidity {
                revocation_status: None,
                validity_window: ValidityWindow::new_valid_mock(),
            },
            &attributes,
            &EmptyPresentationConfig,
        )
        .expect("creating AttestationPresentation should succeed");

        assert_eq!(attestation_presentation.issuer, Organization::new_mock());
        assert_eq!(
            attestation_presentation.attributes,
            vec![
                AttestationAttribute {
                    key: vec_nonempty!["name".to_string()],
                    metadata: vec![ClaimDisplayMetadata {
                        locale: "en".to_string(),
                        label: "name".to_string(),
                        description: None
                    }],
                    value: AttributeValue::Text("Wallet".to_string()),
                    svg_id: None
                },
                AttestationAttribute {
                    key: vec_nonempty!["birth_date".to_string()],
                    metadata: vec![ClaimDisplayMetadata {
                        locale: "en".to_string(),
                        label: "birth date".to_string(),
                        description: None
                    }],
                    value: AttributeValue::Text("1996-06-16".to_owned()),
                    svg_id: None
                },
                AttestationAttribute {
                    key: vec_nonempty!["address".to_string(), "street".to_string()],
                    metadata: vec![ClaimDisplayMetadata {
                        locale: "en".to_string(),
                        label: "address street".to_string(),
                        description: None
                    }],
                    value: AttributeValue::Text("Gracht".to_string()),
                    svg_id: None
                },
                AttestationAttribute {
                    key: vec_nonempty!["address".to_string(), "number".to_string()],
                    metadata: vec![ClaimDisplayMetadata {
                        locale: "en".to_string(),
                        label: "address number".to_string(),
                        description: None
                    }],
                    value: AttributeValue::Integer(123),
                    svg_id: None
                },
            ]
        );
    }

    #[test]
    fn test_create_from_attributes_missing() {
        let attributes = example_attributes();
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"locale": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["name"],
                    "display": [{"locale": "en", "label": "name"}],
                },
                {
                    "path": ["birth_date"],
                    "display": [{"locale": "en", "label": "birth date"}],
                },
            ]
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let error = AttestationPresentation::create_from_attributes(
            AttestationIdentity::Ephemeral,
            Format::SdJwt,
            type_metadata,
            Organization::new_mock(),
            AttestationValidity {
                revocation_status: None,
                validity_window: ValidityWindow::new_valid_mock(),
            },
            &attributes,
            &EmptyPresentationConfig,
        )
        .expect_err("creating AttestationPresentation should not succeed");

        assert_matches!(error, AttestationError::AttributesNotProcessedByClaim(attributes) if attributes ==
        HashSet::from_iter(vec![vec!["address".to_string(), "street".to_string()], vec!["address".to_string(), "number".to_string()]]));
    }

    #[test]
    fn test_filter_recovery_code() {
        let config = test_wallet_config();
        let mdoc_attributes = IndexMap::from([(
            String::from(PID_ATTESTATION_TYPE),
            vec![
                Entry {
                    name: String::from(PID_BSN),
                    value: DataElementValue::Text(String::from("999991772")),
                },
                Entry {
                    name: String::from(PID_RECOVERY_CODE),
                    value: DataElementValue::Text(String::from(
                        "cff292503cba8c4fbf2e5820dcdc468ae00f40c87b1af35513375800128fc00d",
                    )),
                },
            ],
        )]);

        let attestation = AttestationPresentation::create_from_mdoc(
            AttestationIdentity::Ephemeral,
            NormalizedTypeMetadata::nl_pid_example(),
            Organization::new_mock(),
            AttestationValidity {
                revocation_status: None,
                validity_window: ValidityWindow::new_valid_mock(),
            },
            mdoc_attributes,
            &config.pid_attributes,
        )
        .expect("creating AttestationPresentation should succeed");

        let attrs = attestation
            .attributes
            .into_iter()
            .map(|attr| (attr.key, attr.value))
            .collect::<Vec<_>>();

        assert_eq!(
            [(
                vec_nonempty![String::from(PID_BSN)],
                AttributeValue::Text(String::from("999991772"))
            ),],
            attrs.as_slice()
        );
    }
}
