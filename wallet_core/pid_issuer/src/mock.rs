/// Mock implementations of the two traits abstracting other components
use std::{collections::HashMap, ops::Add};

use async_trait::async_trait;
use chrono::{Days, Utc};
use ciborium::Value;
use indexmap::IndexMap;

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    Tdate,
};
use rand::Rng;
use serde::Deserialize;

use crate::{digid, settings::MockAttributes};

use crate::app::{AttributesLookup, BsnLookup};

pub struct MockBsnLookup(Vec<String>);

impl Default for MockBsnLookup {
    fn default() -> Self {
        Self(vec!["999999999".to_owned()])
    }
}

impl From<Vec<MockAttributes>> for MockBsnLookup {
    fn from(value: Vec<MockAttributes>) -> Self {
        Self(value.iter().map(|p| p.person.bsn.clone()).collect())
    }
}

#[async_trait]
impl BsnLookup for MockBsnLookup {
    async fn bsn(&self, _access_token: &str) -> Result<String, digid::Error> {
        Ok(self.0[rand::thread_rng().gen_range(0..self.0.len())].clone())
    }
}

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
        use Gender::*;
        let value = match value {
            Unknown => 0,
            Male => 1,
            Female => 2,
            NotApplicable => 9,
        };
        Value::Integer(value.into())
    }
}

const PID_BSN: &str = "bsn";

const PID_FAMILY_NAME: &str = "family_name";
const PID_GIVEN_NAME: &str = "given_name";
const PID_BIRTH_DATE: &str = "birth_date";
const PID_AGE_OVER_18: &str = "age_over_18";
// const PID_AGE_OVER_NN: &str = "age_over_NN";
// const PID_AGE_IN_YEARS: &str = "age_in_years";
// const PID_AGE_BIRTH_YEAR: &str = "age_birth_year";
const PID_UNIQUE_ID: &str = "unique_id";
const PID_FAMILY_NAME_BIRTH: &str = "family_name_birth";
const PID_GIVEN_NAME_BIRTH: &str = "given_name_birth";
const PID_BIRTH_PLACE: &str = "birth_place";
const PID_BIRTH_COUNTRY: &str = "birth_country";
const PID_BIRTH_STATE: &str = "birth_state";
const PID_BIRTH_CITY: &str = "birth_city";
const PID_RESIDENT_ADDRESS: &str = "resident_address";
const PID_RESIDENT_COUNTRY: &str = "resident_country";
const PID_RESIDENT_STATE: &str = "resident_state";
const PID_RESIDENT_CITY: &str = "resident_city";
const PID_RESIDENT_POSTAL_CODE: &str = "resident_postal_code";
const PID_RESIDENT_STREET: &str = "resident_street";
const PID_RESIDENT_HOUSE_NUMBER: &str = "resident_house_number";
const PID_GENDER: &str = "gender";
const PID_NATIONALITY: &str = "nationality";

#[derive(Default, Deserialize, Clone)]
pub struct PersonAttributes {
    bsn: String,
    family_name: String,
    given_name: String,
    birth_date: chrono::NaiveDate,
    age_over_18: bool,
    // age_over_NN: Option<bool>,
    // age_in_years: Option<u32>,
    // age_birth_year: Option<u32>,
    unique_id: String,
    family_name_birth: Option<String>,
    given_name_birth: Option<String>,
    birth_place: Option<String>,
    birth_country: Option<String>,
    birth_state: Option<String>,
    birth_city: Option<String>,
    gender: Option<Gender>,
    nationality: Option<String>,
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
            Entry {
                name: PID_UNIQUE_ID.to_string(),
                value: Value::Text(value.unique_id),
            }
            .into(),
            value.family_name_birth.map(|v| Entry {
                name: PID_FAMILY_NAME_BIRTH.to_string(),
                value: Value::Text(v),
            }),
            value.given_name_birth.map(|v| Entry {
                name: PID_GIVEN_NAME_BIRTH.to_string(),
                value: Value::Text(v),
            }),
            value.birth_place.map(|v| Entry {
                name: PID_BIRTH_PLACE.to_string(),
                value: Value::Text(v),
            }),
            value.birth_country.map(|v| Entry {
                name: PID_BIRTH_COUNTRY.to_string(),
                // TODO according to ISO 3166-1
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
            value.nationality.map(|v| Entry {
                name: PID_NATIONALITY.to_string(),
                // TODO according to ISO 3166-1
                value: Value::Text(v),
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
            "999999999".to_owned(),
            (
                PersonAttributes {
                    unique_id: "1".to_owned(),
                    bsn: "999999999".to_owned(),
                    family_name: "Van Waarde".to_owned(),
                    given_name: "Johannes Frederik".to_owned(),
                    birth_date: chrono::NaiveDate::parse_from_str("1995-09-21", "%Y-%m-%d").unwrap(),
                    age_over_18: true,
                    ..PersonAttributes::default()
                },
                None,
            ),
        );
        Self(map)
    }
}

impl From<Vec<MockAttributes>> for MockAttributesLookup {
    fn from(value: Vec<MockAttributes>) -> Self {
        Self(
            value
                .iter()
                .map(|p| (p.person.bsn.clone(), (p.person.clone(), p.resident.clone())))
                .collect(),
        )
    }
}

impl AttributesLookup for MockAttributesLookup {
    fn attributes(&self, bsn: &str) -> Vec<UnsignedMdoc> {
        match self.0.get(bsn) {
            Some((person, residence)) => {
                vec![
                    UnsignedMdoc {
                        doc_type: MOCK_PID_DOCTYPE.to_string(),
                        copy_count: 1,
                        valid_from: Tdate::now(),
                        valid_until: Utc::now().add(Days::new(365)).into(),
                        attributes: IndexMap::from([(MOCK_PID_DOCTYPE.to_string(), person.clone().into())]),
                    },
                    UnsignedMdoc {
                        doc_type: MOCK_ADDRESS_DOCTYPE.to_string(),
                        copy_count: 1,
                        valid_from: Tdate::now(),
                        valid_until: Utc::now().add(Days::new(365)).into(),
                        attributes: IndexMap::from([(
                            MOCK_ADDRESS_DOCTYPE.to_string(),
                            residence.clone().unwrap_or_default().into(),
                        )]),
                    },
                ]
            }
            // no attributes in lookup table
            None => vec![],
        }
    }
}
