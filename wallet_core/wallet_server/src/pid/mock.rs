/// Mock implementations of the two traits abstracting other components
use std::collections::HashMap;

use chrono::NaiveDate;
use serde::Deserialize;

use openid4vc::attributes::Attribute;
use openid4vc::attributes::AttributeValue;
use openid4vc::attributes::IssuableDocument;
use openid4vc::attributes::IssuableDocuments;

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

impl From<Gender> for AttributeValue {
    fn from(value: Gender) -> Self {
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
        AttributeValue::Number(value as isize)
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

impl From<PersonAttributes> for IssuableDocument {
    fn from(value: PersonAttributes) -> Self {
        Self::try_new(
            MOCK_PID_DOCTYPE.to_string(),
            vec![
                (PID_BSN.to_string(), Attribute::Single(AttributeValue::Text(value.bsn))).into(),
                (
                    PID_FAMILY_NAME.to_string(),
                    Attribute::Single(AttributeValue::Text(value.family_name)),
                )
                    .into(),
                (
                    PID_GIVEN_NAME.to_string(),
                    Attribute::Single(AttributeValue::Text(value.given_name)),
                )
                    .into(),
                (
                    PID_BIRTH_DATE.to_string(),
                    Attribute::Single(AttributeValue::Text(value.birth_date.format("%Y-%m-%d").to_string())),
                )
                    .into(),
                (
                    PID_AGE_OVER_18.to_string(),
                    Attribute::Single(AttributeValue::Bool(value.age_over_18)),
                )
                    .into(),
                value.birth_country.map(|v| {
                    (
                        PID_BIRTH_COUNTRY.to_string(),
                        Attribute::Single(AttributeValue::Text(v)),
                    )
                }),
                value
                    .birth_state
                    .map(|v| (PID_BIRTH_STATE.to_string(), Attribute::Single(AttributeValue::Text(v)))),
                value
                    .birth_city
                    .map(|v| (PID_BIRTH_CITY.to_string(), Attribute::Single(AttributeValue::Text(v)))),
                value
                    .gender
                    .map(|v| (PID_GENDER.to_string(), Attribute::Single(v.into()))),
            ]
            .into_iter()
            .flatten()
            .collect(),
        )
        .unwrap()
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

impl From<ResidentAttributes> for IssuableDocument {
    fn from(value: ResidentAttributes) -> Self {
        Self::try_new(
            MOCK_ADDRESS_DOCTYPE.to_string(),
            vec![
                value.address.map(|v| {
                    (
                        PID_RESIDENT_ADDRESS.to_string(),
                        Attribute::Single(AttributeValue::Text(v)),
                    )
                }),
                value.country.map(|v| {
                    (
                        PID_RESIDENT_COUNTRY.to_string(),
                        Attribute::Single(AttributeValue::Text(v)),
                    )
                }),
                value.state.map(|v| {
                    (
                        PID_RESIDENT_STATE.to_string(),
                        Attribute::Single(AttributeValue::Text(v)),
                    )
                }),
                value.city.map(|v| {
                    (
                        PID_RESIDENT_CITY.to_string(),
                        Attribute::Single(AttributeValue::Text(v)),
                    )
                }),
                value.postal_code.map(|v| {
                    (
                        PID_RESIDENT_POSTAL_CODE.to_string(),
                        Attribute::Single(AttributeValue::Text(v)),
                    )
                }),
                value.street.map(|v| {
                    (
                        PID_RESIDENT_STREET.to_string(),
                        Attribute::Single(AttributeValue::Text(v)),
                    )
                }),
                value.house_number.map(|v| {
                    (
                        PID_RESIDENT_HOUSE_NUMBER.to_string(),
                        Attribute::Single(AttributeValue::Text(v)),
                    )
                }),
            ]
            .into_iter()
            .flatten()
            .collect(),
        )
        .unwrap()
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
    pub fn attributes(&self, bsn: &str) -> Option<IssuableDocuments> {
        let (person, residence) = self.0.get(bsn)?;

        let attrs = vec![Some(person.to_owned().into()), residence.to_owned().map(|r| r.into())]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Some(attrs)
    }
}
