use chrono::Days;
use chrono::Utc;
use indexmap::IndexMap;

use attestation_data::attributes::Entry;
use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
use mdoc::unsigned::UnsignedMdoc;
use mdoc::DataElementValue;
use sd_jwt_vc_metadata::JsonSchemaPropertyFormat;
use sd_jwt_vc_metadata::JsonSchemaPropertyType;
use sd_jwt_vc_metadata::TypeMetadata;

use super::BSN_ATTR_NAME;
use super::PID_DOCTYPE;

fn create_empty_unsigned_mdoc() -> UnsignedMdoc {
    let now = Utc::now();

    UnsignedMdoc {
        doc_type: PID_DOCTYPE.to_string(),
        valid_from: now.into(),
        valid_until: (now + Days::new(365)).into(),
        attributes: IndexMap::from([(
            PID_DOCTYPE.to_string(),
            vec![Entry {
                name: "dummy".to_string(),
                value: DataElementValue::Text("foo".to_string()),
            }],
        )])
        .try_into()
        .unwrap(),
        issuer_uri: format!("https://{ISSUANCE_CERT_CN}").parse().unwrap(),
        attestation_qualification: Default::default(),
    }
}

pub fn create_bsn_only_unsigned_mdoc() -> (UnsignedMdoc, TypeMetadata) {
    let mut unsigned_mdoc = create_empty_unsigned_mdoc();

    unsigned_mdoc.attributes = IndexMap::from([(
        PID_DOCTYPE.to_string(),
        vec![Entry {
            name: BSN_ATTR_NAME.to_string(),
            value: DataElementValue::Text("999999999".to_string()),
        }],
    )])
    .try_into()
    .unwrap();

    let metadata =
        TypeMetadata::example_with_claim_name(PID_DOCTYPE, BSN_ATTR_NAME, JsonSchemaPropertyType::String, None);

    (unsigned_mdoc, metadata)
}

pub fn create_example_unsigned_mdoc() -> (UnsignedMdoc, TypeMetadata) {
    let mut unsigned_mdoc = create_empty_unsigned_mdoc();

    unsigned_mdoc.attributes = IndexMap::from([(
        PID_DOCTYPE.to_string(),
        vec![
            Entry {
                name: "family_name".to_string(),
                value: DataElementValue::Text("De Bruijn".to_string()),
            },
            Entry {
                name: "given_name".to_string(),
                value: DataElementValue::Text("Willeke Liselotte".to_string()),
            },
            Entry {
                name: "birth_date".to_string(),
                value: DataElementValue::Text("1997-05-10".to_string()),
            },
            Entry {
                name: "age_over_18".to_string(),
                value: DataElementValue::Bool(true),
            },
        ],
    )])
    .try_into()
    .unwrap();

    let metadata = TypeMetadata::example_with_claim_names(
        PID_DOCTYPE,
        &[
            ("family_name", JsonSchemaPropertyType::String, None),
            ("given_name", JsonSchemaPropertyType::String, None),
            (
                "birth_date",
                JsonSchemaPropertyType::String,
                Some(JsonSchemaPropertyFormat::Date),
            ),
            ("age_over_18", JsonSchemaPropertyType::Boolean, None),
        ],
    );

    (unsigned_mdoc, metadata)
}
