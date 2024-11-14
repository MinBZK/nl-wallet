use std::collections::HashMap;
use std::sync::LazyLock;

use indexmap::IndexMap;

use super::mdoc::AttributeValueType;
use super::AttributeKey;
use super::AttributeLabels;
use super::ADDRESS_DOCTYPE;
use super::PID_DOCTYPE;

#[derive(Debug, Clone)]
pub(super) struct DataElementValueMapping {
    pub key: AttributeKey,
    pub is_mandatory: bool,
    pub key_labels: AttributeLabels,
    pub value_type: AttributeValueType,
}

pub(super) type MappingNameSpace = &'static str;
pub(super) type MappingDataElementIdentifier = &'static str;
pub(super) type AttributeMapping = IndexMap<(MappingNameSpace, MappingDataElementIdentifier), DataElementValueMapping>;

pub(super) type MappingDocType = &'static str;
pub(super) type MdocDocumentMapping = HashMap<MappingDocType, AttributeMapping>;

pub(super) static MDOC_DOCUMENT_MAPPING: LazyLock<MdocDocumentMapping> = LazyLock::new(|| {
    HashMap::from([
        (
            PID_DOCTYPE,
            IndexMap::from([
                (
                    (PID_DOCTYPE, "given_name"),
                    DataElementValueMapping {
                        key: "given_name",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "First names"), ("nl", "Voornamen")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (PID_DOCTYPE, "family_name"),
                    DataElementValueMapping {
                        key: "family_name",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "Surname"), ("nl", "Achternaam")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (PID_DOCTYPE, "given_name_birth"),
                    DataElementValueMapping {
                        key: "given_name_birth",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "First names at birth"), ("nl", "Voornamen bij geboorte")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (PID_DOCTYPE, "family_name_birth"),
                    DataElementValueMapping {
                        key: "family_name_birth",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Birth name"), ("nl", "Geboortenaam")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (PID_DOCTYPE, "gender"),
                    DataElementValueMapping {
                        key: "gender",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Gender"), ("nl", "Geslacht")]),
                        value_type: AttributeValueType::Gender,
                    },
                ),
                (
                    (PID_DOCTYPE, "birth_date"),
                    DataElementValueMapping {
                        key: "birth_date",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "Birth date"), ("nl", "Geboortedatum")]),
                        value_type: AttributeValueType::Date,
                    },
                ),
                (
                    (PID_DOCTYPE, "age_over_18"),
                    DataElementValueMapping {
                        key: "age_over_18",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "Older than 18"), ("nl", "Ouder dan 18")]),
                        value_type: AttributeValueType::Bool,
                    },
                ),
                (
                    (PID_DOCTYPE, "birth_place"),
                    DataElementValueMapping {
                        key: "birth_place",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Place of birth"), ("nl", "Geboorteplaats")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (PID_DOCTYPE, "birth_city"),
                    DataElementValueMapping {
                        key: "birth_city",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "City, town or village of birth"), ("nl", "Geboortestad")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (PID_DOCTYPE, "birth_state"),
                    DataElementValueMapping {
                        key: "birth_state",
                        is_mandatory: false,
                        key_labels: HashMap::from([
                            ("en", "State or province of birth"),
                            ("nl", "Geboortestaat of -provincie"),
                        ]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (PID_DOCTYPE, "birth_country"),
                    DataElementValueMapping {
                        key: "birth_country",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Country of birth"), ("nl", "Geboorteland")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (PID_DOCTYPE, "bsn"),
                    DataElementValueMapping {
                        key: "bsn",
                        is_mandatory: true,
                        key_labels: HashMap::from([("en", "BSN"), ("nl", "BSN")]),
                        value_type: AttributeValueType::String,
                    },
                ),
            ]),
        ),
        (
            ADDRESS_DOCTYPE,
            IndexMap::from([
                (
                    (ADDRESS_DOCTYPE, "resident_address"),
                    DataElementValueMapping {
                        key: "resident_address",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Address"), ("nl", "Adres")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (ADDRESS_DOCTYPE, "resident_street"),
                    DataElementValueMapping {
                        key: "resident_street",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Street"), ("nl", "Straatnaam")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (ADDRESS_DOCTYPE, "resident_house_number"),
                    DataElementValueMapping {
                        key: "resident_house_number",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "House number"), ("nl", "Huisnummer")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (ADDRESS_DOCTYPE, "resident_postal_code"),
                    DataElementValueMapping {
                        key: "resident_postal_code",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Postal code"), ("nl", "Postcode")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (ADDRESS_DOCTYPE, "resident_city"),
                    DataElementValueMapping {
                        key: "resident_city",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "City, town or village"), ("nl", "Woonplaats")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (ADDRESS_DOCTYPE, "resident_state"),
                    DataElementValueMapping {
                        key: "resident_state",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "State or province"), ("nl", "Staat of provincie")]),
                        value_type: AttributeValueType::String,
                    },
                ),
                (
                    (ADDRESS_DOCTYPE, "resident_country"),
                    DataElementValueMapping {
                        key: "resident_country",
                        is_mandatory: false,
                        key_labels: HashMap::from([("en", "Country"), ("nl", "Land")]),
                        value_type: AttributeValueType::String,
                    },
                ),
            ]),
        ),
    ])
});
