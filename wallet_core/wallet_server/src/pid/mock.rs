/// Mock implementations of the two traits abstracting other components
use std::collections::HashMap;
/// Mock implementations of the two traits abstracting other components
use std::num::NonZeroU8;
/// Mock implementations of the two traits abstracting other components
use std::ops::Add;

use chrono::Days;
use chrono::NaiveDate;
use chrono::Utc;
use ciborium::Value;
use indexmap::IndexMap;
use serde::Deserialize;

use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::unsigned::UnsignedMdoc;
use nl_wallet_mdoc::Tdate;

use crate::pid::constants::*;

// ISO/IEC 5218
#[allow(dead_code)]
#[derive(Deserialize, Clone)]
enum Gender {
    Unknown,
    Male,
    Female,
    NotApplicable,
}

impl From<Gender> for Value {
    fn from(value: Gender) -> Value {
        use Gender::Female;
        use Gender::Male;
        use Gender::NotApplicable;
        use Gender::Unknown;
        let value = match value {
            Unknown => 0,
            Male => 1,
            Female => 2,
            NotApplicable => 9,
        };
        Value::Integer(value.into())
    }
}

#[derive(Default, Deserialize, Clone)]
pub struct PersonAttributes {
    bsn: String,
    family_name: String,
    given_name: String,
    birth_date: NaiveDate,
    age_over_18: bool,
    // age_over_NN: Option<bool>,
    // age_in_years: Option<u32>,
    // age_birth_year: Option<u32>,
    birth_country: Option<String>,
    birth_state: Option<String>,
    birth_city: Option<String>,
    gender: Option<Gender>,
}

impl From<PersonAttributes> for Vec<Entry> {
    fn from(value: PersonAttributes) -> Vec<Entry> {
        vec![
            Entry {
                name: PID_BSN.to_string(),
                value: Value::Text(value.bsn),
            }
            .into(),
            Entry {
                name: PID_FAMILY_NAME.to_string(),
                value: Value::Text(value.family_name),
            }
            .into(),
            Entry {
                name: PID_GIVEN_NAME.to_string(),
                value: Value::Text(value.given_name),
            }
            .into(),
            Entry {
                name: PID_BIRTH_DATE.to_string(),
                value: Value::Text(value.birth_date.format("%Y-%m-%d").to_string()),
            }
            .into(),
            Entry {
                name: PID_AGE_OVER_18.to_string(),
                value: Value::Bool(value.age_over_18),
            }
            .into(),
            value.birth_country.map(|v| Entry {
                name: PID_BIRTH_COUNTRY.to_string(),
                value: Value::Text(v),
            }),
            value.birth_state.map(|v| Entry {
                name: PID_BIRTH_STATE.to_string(),
                value: Value::Text(v),
            }),
            value.birth_city.map(|v| Entry {
                name: PID_BIRTH_CITY.to_string(),
                value: Value::Text(v),
            }),
            value.gender.map(|v| Entry {
                name: PID_GENDER.to_string(),
                value: v.into(),
            }),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[derive(Default, Deserialize, Clone)]
pub struct ResidentAttributes {
    address: Option<String>,
    country: Option<String>,
    state: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
    street: Option<String>,
    house_number: Option<String>,
}

impl From<ResidentAttributes> for Vec<Entry> {
    fn from(value: ResidentAttributes) -> Vec<Entry> {
        vec![
            value.address.map(|v| Entry {
                name: PID_RESIDENT_ADDRESS.to_string(),
                value: Value::Text(v),
            }),
            value.country.map(|v| Entry {
                name: PID_RESIDENT_COUNTRY.to_string(),
                value: Value::Text(v),
            }),
            value.state.map(|v| Entry {
                name: PID_RESIDENT_STATE.to_string(),
                value: Value::Text(v),
            }),
            value.city.map(|v| Entry {
                name: PID_RESIDENT_CITY.to_string(),
                value: Value::Text(v),
            }),
            value.postal_code.map(|v| Entry {
                name: PID_RESIDENT_POSTAL_CODE.to_string(),
                value: Value::Text(v),
            }),
            value.street.map(|v| Entry {
                name: PID_RESIDENT_STREET.to_string(),
                value: Value::Text(v),
            }),
            value.house_number.map(|v| Entry {
                name: PID_RESIDENT_HOUSE_NUMBER.to_string(),
                value: Value::Text(v),
            }),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

const MOCK_PID_DOCTYPE: &str = "com.example.pid";
const MOCK_ADDRESS_DOCTYPE: &str = "com.example.address";

type Attributes = (PersonAttributes, Option<ResidentAttributes>);
pub struct MockAttributesLookup(HashMap<String, Attributes>);

impl Default for MockAttributesLookup {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(
            "999991772".to_owned(),
            (
                PersonAttributes {
                    bsn: "999991772".to_owned(),
                    given_name: "Willeke Liselotte".to_owned(),
                    family_name: "De Bruijn".to_owned(),
                    gender: Some(Gender::Female),
                    birth_date: NaiveDate::parse_from_str("1997-05-10", "%Y-%m-%d").unwrap(),
                    age_over_18: true,
                    birth_country: Some("NL".to_owned()),
                    birth_city: Some("Delft".to_owned()),
                    birth_state: Some("Zuid-Holland".to_owned()),
                },
                Some(ResidentAttributes {
                    street: Some("Turfmarkt".to_owned()),
                    house_number: Some("147".to_owned()),
                    postal_code: Some("2511 DP".to_owned()),
                    city: Some("Den Haag".to_owned()),
                    state: Some("Zuid-Holland".to_owned()),
                    country: Some("NL".to_owned()),
                    ..ResidentAttributes::default()
                }),
            ),
        );
        Self(map)
    }
}

impl MockAttributesLookup {
    pub fn attributes(&self, bsn: &str) -> Option<Vec<UnsignedMdoc>> {
        let (person, residence) = self.0.get(bsn)?;

        let attrs = vec![
            Some(UnsignedMdoc {
                doc_type: MOCK_PID_DOCTYPE.to_string(),
                copy_count: NonZeroU8::new(2).unwrap(),
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(MOCK_PID_DOCTYPE.to_string(), person.clone().into())])
                    .try_into()
                    .unwrap(),
            }),
            residence
                .as_ref()
                .and_then(|residence| {
                    // This will return `None` if the `UnsignedAttributes` is empty.
                    IndexMap::from([(MOCK_ADDRESS_DOCTYPE.to_string(), residence.clone().into())])
                        .try_into()
                        .ok()
                })
                .map(|attributes| UnsignedMdoc {
                    doc_type: MOCK_ADDRESS_DOCTYPE.to_string(),
                    copy_count: NonZeroU8::new(2).unwrap(),
                    valid_from: Tdate::now(),
                    valid_until: Utc::now().add(Days::new(365)).into(),
                    attributes,
                }),
        ]
        .into_iter()
        .flatten()
        .collect();

        Some(attrs)
    }
}
