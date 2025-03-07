use std::num::NonZeroU8;

use chrono::Days;
use chrono::Utc;
use indexmap::IndexMap;

use nl_wallet_mdoc::server_keys::generate::mock::ISSUANCE_CERT_CN;
use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::unsigned::UnsignedMdoc;
use nl_wallet_mdoc::DataElementValue;
use sd_jwt::metadata::JsonSchemaPropertyFormat;
use sd_jwt::metadata::JsonSchemaPropertyType;
use sd_jwt::metadata::TypeMetadata;

use super::PID_DOCTYPE;

pub fn create_bsn_only_unsigned_mdoc() -> (UnsignedMdoc, TypeMetadata) {
    let now = Utc::now();

    (
        UnsignedMdoc {
            doc_type: PID_DOCTYPE.to_string(),
            copy_count: NonZeroU8::new(1).unwrap(),
            valid_from: now.into(),
            valid_until: (now + Days::new(365)).into(),
            attributes: IndexMap::from([(
                PID_DOCTYPE.to_string(),
                vec![Entry {
                    name: "bsn".to_string(),
                    value: DataElementValue::Text("999999999".to_string()),
                }],
            )])
            .try_into()
            .unwrap(),
            issuer_uri: format!("https://{ISSUANCE_CERT_CN}").parse().unwrap(),
        },
        TypeMetadata::example_with_claim_name(PID_DOCTYPE, "bsn", JsonSchemaPropertyType::String, None),
    )
}

pub fn create_example_unsigned_mdoc() -> (UnsignedMdoc, TypeMetadata) {
    let (mut unsigned_mdoc, _type_metadata) = create_bsn_only_unsigned_mdoc();
    let mut attributes = unsigned_mdoc.attributes.into_inner();

    attributes.get_mut(PID_DOCTYPE).unwrap().extend(vec![
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
    ]);

    let metadata = TypeMetadata::example_with_claim_names(
        PID_DOCTYPE,
        &[
            ("bsn", JsonSchemaPropertyType::String, None),
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

    unsigned_mdoc.attributes = attributes.try_into().unwrap();

    (unsigned_mdoc, metadata)
}
