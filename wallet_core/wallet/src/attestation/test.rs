use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use p256::ecdsa::SigningKey;
use rand_core::OsRng;

use attestation_data::attributes::AttributeValue;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use mdoc::Entry;
use sd_jwt_vc_metadata::JsonSchemaPropertyFormat;
use sd_jwt_vc_metadata::JsonSchemaPropertyType;
use sd_jwt_vc_metadata::TypeMetadata;
use utils::generator::Generator;

use crate::attestation::BSN_ATTR_NAME;
use crate::attestation::PID_DOCTYPE;

pub fn create_bsn_only_payload_preview(
    time_generator: &impl Generator<DateTime<Utc>>,
) -> (PreviewableCredentialPayload, TypeMetadata) {
    let payload = CredentialPayload::example_with_attributes(
        vec![("bsn", AttributeValue::Text("999999999".to_string()))],
        SigningKey::random(&mut OsRng).verifying_key(),
        time_generator,
    );

    let metadata =
        TypeMetadata::example_with_claim_name(PID_DOCTYPE, BSN_ATTR_NAME, JsonSchemaPropertyType::String, None);

    (payload.previewable_payload, metadata)
}

// TODO: Remove this when all tests use `PreviewableCredentialPayload`
pub fn create_bsn_only_mdoc_attributes(
    time_generator: &impl Generator<DateTime<Utc>>,
) -> (IndexMap<String, Vec<Entry>>, TypeMetadata) {
    let (payload, metadata) = create_bsn_only_payload_preview(time_generator);

    (
        payload.attributes.to_mdoc_attributes(&payload.attestation_type),
        metadata,
    )
}

// NOTE: this example and metadata should comply with "eudi:pid:nl:1.json"
pub fn create_example_payload_preview(
    time_generator: &impl Generator<DateTime<Utc>>,
) -> (PreviewableCredentialPayload, TypeMetadata) {
    let payload = CredentialPayload::example_with_attributes(
        vec![
            ("family_name", AttributeValue::Text("De Bruijn".to_string())),
            ("given_name", AttributeValue::Text("Willeke Liselotte".to_string())),
            ("birthdate", AttributeValue::Text("1997-05-10".to_string())),
            ("age_over_18", AttributeValue::Bool(true)),
        ],
        SigningKey::random(&mut OsRng).verifying_key(),
        time_generator,
    );

    let metadata = TypeMetadata::example_with_claim_names(
        PID_DOCTYPE,
        &[
            ("family_name", JsonSchemaPropertyType::String, None),
            ("given_name", JsonSchemaPropertyType::String, None),
            (
                "birthdate",
                JsonSchemaPropertyType::String,
                Some(JsonSchemaPropertyFormat::Date),
            ),
            ("age_over_18", JsonSchemaPropertyType::Boolean, None),
        ],
    );

    (payload.previewable_payload, metadata)
}

// TODO: Remove this when all tests use `PreviewableCredentialPayload`
pub fn create_example_mdoc_attributes(
    time_generator: &impl Generator<DateTime<Utc>>,
) -> (IndexMap<String, Vec<Entry>>, TypeMetadata) {
    let (payload, metadata) = create_example_payload_preview(time_generator);

    (
        payload.attributes.to_mdoc_attributes(&payload.attestation_type),
        metadata,
    )
}
