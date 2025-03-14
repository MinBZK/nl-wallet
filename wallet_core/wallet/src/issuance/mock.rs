use std::num::NonZeroU8;

use chrono::Days;
use chrono::Utc;
use indexmap::IndexMap;

use mdoc::server_keys::generate::mock::ISSUANCE_CERT_CN;
use mdoc::unsigned::Entry;
use mdoc::unsigned::UnsignedMdoc;
use mdoc::DataElementValue;
use sd_jwt::metadata::TypeMetadata;

use super::PID_DOCTYPE;

/// This creates an `UnsignedMdoc` that only contains a bsn entry.
pub fn create_bsn_only_unsigned_pid_mdoc() -> (UnsignedMdoc, TypeMetadata) {
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
        TypeMetadata::example_with_claim_name("bsn"),
    )
}

/// This creates a minimal `UnsignedMdoc` that is valid.
pub fn create_minimal_unsigned_pid_mdoc() -> (UnsignedMdoc, TypeMetadata) {
    let (mut unsigned_mdoc, mut metadata) = create_bsn_only_unsigned_pid_mdoc();
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

    unsigned_mdoc.attributes = attributes.try_into().unwrap();

    metadata.claims.extend(
        TypeMetadata::example_with_claim_names(&["family_name", "given_name", "birth_date", "age_over_18"]).claims,
    );

    (unsigned_mdoc, metadata)
}

/// This creates a full `UnsignedMdoc` that is valid.
pub fn create_full_unsigned_pid_mdoc() -> (UnsignedMdoc, TypeMetadata) {
    let (mut unsigned_mdoc, mut metadata) = create_minimal_unsigned_pid_mdoc();
    let mut attributes = unsigned_mdoc.attributes.into_inner();

    attributes.get_mut(PID_DOCTYPE).unwrap().extend(vec![
        Entry {
            name: "family_name_birth".to_string(),
            value: DataElementValue::Text("Molenaar".to_string()),
        },
        Entry {
            name: "given_name_birth".to_string(),
            value: DataElementValue::Text("Liselotte Willeke".to_string()),
        },
        Entry {
            name: "birth_place".to_string(),
            value: DataElementValue::Text("Delft".to_string()),
        },
        Entry {
            name: "birth_country".to_string(),
            value: DataElementValue::Text("NL".to_string()),
        },
        Entry {
            name: "birth_state".to_string(),
            value: DataElementValue::Text("Zuid-Holland".to_string()),
        },
        Entry {
            name: "birth_city".to_string(),
            value: DataElementValue::Text("Delft".to_string()),
        },
        Entry {
            name: "gender".to_string(),
            value: DataElementValue::Integer(2.into()),
        },
    ]);

    unsigned_mdoc.attributes = attributes.try_into().unwrap();

    metadata.claims.extend(
        TypeMetadata::example_with_claim_names(&[
            "family_name_birth",
            "given_name_birth",
            "birth_place",
            "birth_country",
            "birth_state",
            "birth_city",
            "gender",
        ])
        .claims,
    );

    (unsigned_mdoc, metadata)
}
