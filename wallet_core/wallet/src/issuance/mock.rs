use indexmap::IndexMap;
use p256::ecdsa::SigningKey;
use rand_core::OsRng;

use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Entry;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use mdoc::DataElementValue;
use sd_jwt_vc_metadata::JsonSchemaPropertyFormat;
use sd_jwt_vc_metadata::JsonSchemaPropertyType;
use sd_jwt_vc_metadata::TypeMetadata;

use super::PID_DOCTYPE;

pub fn create_bsn_only_mdoc_attributes() -> (IndexMap<String, Vec<Entry>>, TypeMetadata) {
    let attributes = IndexMap::from([(
        PID_DOCTYPE.to_string(),
        vec![Entry {
            name: "bsn".to_string(),
            value: DataElementValue::Text("999999999".to_string()),
        }],
    )]);

    let metadata = TypeMetadata::example_with_claim_name(PID_DOCTYPE, "bsn", JsonSchemaPropertyType::String, None);

    (attributes, metadata)
}

pub fn create_example_mdoc_attributes() -> (IndexMap<String, Vec<Entry>>, TypeMetadata) {
    let attributes = IndexMap::from([(
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
    )]);

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

    (attributes, metadata)
}

pub fn create_bsn_only_payload_preview() -> (PreviewableCredentialPayload, TypeMetadata) {
    let payload = CredentialPayload::example_with_attributes(
        vec![("bsn", AttributeValue::Text("999999999".to_string()))],
        SigningKey::random(&mut OsRng).verifying_key(),
    );

    let metadata = TypeMetadata::example_with_claim_name(PID_DOCTYPE, "bsn", JsonSchemaPropertyType::String, None);

    (payload.previewable_payload, metadata)
}

pub fn create_example_payload_preview() -> (PreviewableCredentialPayload, TypeMetadata) {
    let payload = CredentialPayload::example_with_attributes(
        vec![
            ("family_name", AttributeValue::Text("De Bruijn".to_string())),
            ("given_name", AttributeValue::Text("Willeke Liselotte".to_string())),
            ("birth_date", AttributeValue::Text("1997-05-10".to_string())),
            ("age_over_18", AttributeValue::Bool(true)),
        ],
        SigningKey::random(&mut OsRng).verifying_key(),
    );

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

    (payload.previewable_payload, metadata)
}
