use chrono::NaiveDate;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde_with::skip_serializing_none;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use utils::vec_at_least::VecNonEmpty;

use crate::pid::constants::*;

#[derive(Deserialize)]
pub struct BrpPersons {
    #[serde(rename = "personen")]
    pub persons: Vec<BrpPerson>,
}

// Represents a person from the BRP.
// Note: for categories that can occur multiple times, the ordering is such that the most recent category is first.
// See Logisch Ontwerp BRP 2024 Q2 section 5.1.7.3
#[derive(Deserialize)]
pub struct BrpPerson {
    #[serde(rename = "burgerservicenummer")]
    bsn: String,

    #[expect(dead_code)]
    #[serde(rename = "geslacht")]
    gender: Option<BrpGender>,

    #[serde(rename = "naam")]
    name: BrpName,

    #[serde(rename = "geboorte")]
    birth: BrpBirth,

    #[serde(rename = "leeftijd")]
    age: u8,

    #[serde(rename = "nationaliteiten", default, skip_serializing_if = "Vec::is_empty")]
    nationalities: Vec<BrpNationality>,

    #[serde(rename = "verblijfplaats")]
    residence: BrpResidence,
}

impl BrpPerson {
    fn is_over_18(&self) -> bool {
        self.age >= 18
    }
}

impl BrpPerson {
    pub fn into_issuable(self) -> VecNonEmpty<(String, IndexMap<String, Attribute>)> {
        let given_names = self.name.given_names.clone();
        let is_over_18 = self.is_over_18();
        let family_name = self.name.into_name_with_prefix();
        let street = self.residence.address.street().map(String::from);
        let house_number = self.residence.address.locator_designator();
        let nationalities = self
            .nationalities
            .into_iter()
            .filter_map(|nationality| nationality.nationality.map(|nationality| nationality.description))
            .collect_vec();

        vec![
            (
                String::from(PID_ATTESTATION_TYPE),
                IndexMap::from_iter(
                    vec![
                        Some((
                            String::from(PID_FAMILY_NAME),
                            Attribute::Single(AttributeValue::Text(family_name)),
                        )),
                        given_names.map(|names| {
                            (
                                String::from(PID_GIVEN_NAME),
                                Attribute::Single(AttributeValue::Text(names)),
                            )
                        }),
                        Some((
                            String::from(PID_BIRTH_DATE),
                            Attribute::Single(AttributeValue::Text(
                                self.birth.date.date.format("%Y-%m-%d").to_string(),
                            )),
                        )),
                        Some((
                            String::from(PID_AGE_OVER_18),
                            Attribute::Single(AttributeValue::Bool(is_over_18)),
                        )),
                        Some((String::from(PID_BSN), Attribute::Single(AttributeValue::Text(self.bsn)))),
                        Some((
                            String::from(PID_NATIONALITY),
                            Attribute::Single(AttributeValue::Array(
                                nationalities.into_iter().map(AttributeValue::Text).collect_vec(),
                            )),
                        )),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(String, Attribute)>>(),
                ),
            ),
            (
                String::from(ADDRESS_ATTESTATION_TYPE),
                IndexMap::from_iter(vec![(
                    String::from(PID_ADDRESS_GROUP),
                    Attribute::Nested(IndexMap::from_iter(
                        vec![
                            street.map(|street| {
                                (
                                    String::from(PID_RESIDENT_STREET),
                                    Attribute::Single(AttributeValue::Text(street)),
                                )
                            }),
                            Some((
                                String::from(PID_RESIDENT_HOUSE_NUMBER),
                                Attribute::Single(AttributeValue::Text(house_number)),
                            )),
                            Some((
                                String::from(PID_RESIDENT_POSTAL_CODE),
                                Attribute::Single(AttributeValue::Text(self.residence.address.postal_code)),
                            )),
                            Some((
                                String::from(PID_RESIDENT_CITY),
                                Attribute::Single(AttributeValue::Text(self.residence.address.city)),
                            )),
                            Some((
                                String::from(PID_RESIDENT_COUNTRY),
                                Attribute::Single(AttributeValue::Text(self.residence.address.country.description)),
                            )),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<(String, Attribute)>>(),
                    )),
                )]),
            ),
        ]
        .try_into()
        .unwrap()
    }
}

#[derive(Deserialize)]
pub struct BrpGender {
    #[expect(dead_code)]
    code: BrpGenderCode,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all(deserialize = "UPPERCASE"))]
pub enum BrpGenderCode {
    V,
    M,
    O,
}

impl From<BrpGenderCode> for Attribute {
    fn from(value: BrpGenderCode) -> Attribute {
        let value = match value {
            BrpGenderCode::O => 0,
            BrpGenderCode::M => 1,
            BrpGenderCode::V => 2,
        };
        Attribute::Single(AttributeValue::Integer(value))
    }
}

#[derive(Clone, Deserialize)]
pub struct BrpName {
    #[serde(rename = "voornamen")]
    given_names: Option<String>,

    #[serde(rename = "voorvoegsel")]
    family_name_prefix: Option<String>,

    #[serde(rename = "geslachtsnaam")]
    family_name: String,
}

impl BrpName {
    fn into_name_with_prefix(self) -> String {
        if let Some(prefix) = self.family_name_prefix {
            format!("{} {}", prefix, self.family_name)
        } else {
            self.family_name
        }
    }
}

#[derive(Deserialize)]
pub struct BrpBirth {
    #[serde(rename = "datum")]
    date: BrpDate,

    #[expect(dead_code)]
    #[serde(rename = "land")]
    country: Option<BrpDescription>,

    #[expect(dead_code)]
    #[serde(rename = "plaats")]
    place: Option<BrpDescription>,
}

#[derive(Deserialize)]
pub struct BrpDate {
    #[serde(rename = "datum")]
    date: NaiveDate,
}

#[derive(Clone, Deserialize)]
pub struct BrpDescription {
    #[serde(rename = "omschrijving")]
    description: String,
}

#[skip_serializing_none]
#[derive(Deserialize, Clone)]
pub struct BrpNationality {
    #[serde(rename = "nationaliteit")]
    nationality: Option<BrpDescription>,
}

#[derive(Deserialize)]
pub struct BrpResidence {
    #[serde(rename = "verblijfadres")]
    address: BrpAddress,
}

#[derive(Deserialize)]
pub struct BrpAddress {
    #[serde(rename = "officieleStraatnaam")]
    official_street_name: Option<String>,

    #[serde(rename = "korteStraatnaam")]
    short_street_name: Option<String>,

    #[serde(rename = "huisnummer")]
    house_number: u32,

    #[serde(rename = "huisletter")]
    house_letter: Option<String>,

    #[serde(rename = "huisnummertoevoeging")]
    house_number_addition: Option<String>,

    #[serde(rename = "aanduidingbijHuisnummer")]
    house_number_designation: Option<BrpDescription>,

    #[serde(rename = "postcode")]
    postal_code: String,

    #[serde(rename = "woonplaats")]
    city: String,

    #[serde(rename = "land", default = "default_country")]
    country: BrpDescription,
}

fn default_country() -> BrpDescription {
    BrpDescription {
        description: String::from("Nederland"),
    }
}

impl BrpAddress {
    fn street(&self) -> Option<&str> {
        self.official_street_name
            .as_deref()
            .or(self.short_street_name.as_deref())
    }

    fn locator_designator(&self) -> String {
        let house_letter = self.house_letter.as_deref().unwrap_or_default();
        let house_number_addition = self.house_number_addition.as_deref().unwrap_or_default();

        // According to Logisch Ontwerp BRP 2024 Q2, elements 11.40 and 11.50 are mutually exclusive.
        // This implementation is according to "Bijlage 3 Vertaaltabel" describing the GBA-V translations regarding
        // the EAD eIDAS attributes.
        if let Some(designation) = &self.house_number_designation {
            format!("{} {}{}", designation.description, self.house_number, house_letter)
        } else {
            format!("{}{}{}", self.house_number, house_letter, house_number_addition)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rstest::rstest;

    use serde_json::json;

    use utils::path::prefix_local_path;

    use crate::pid::brp::data::BrpPersons;
    use crate::pid::constants::ADDRESS_ATTESTATION_TYPE;
    use crate::pid::constants::PID_ATTESTATION_TYPE;

    fn read_json(name: &str) -> String {
        fs::read_to_string(prefix_local_path(
            format!("resources/test/haal-centraal-examples/{name}.json").as_ref(),
        ))
        .unwrap()
    }

    #[test]
    fn should_deserialize_empty_response() {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json("empty")).unwrap();
        assert!(brp_persons.persons.is_empty());
    }

    #[test]
    fn should_be_over_18() {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert!(brp_person.is_over_18());
    }

    #[rstest]
    #[case("missing-bsn")]
    #[case("missing-family-name")]
    #[case("buitenlands-adres")]
    fn should_error_in_missing_bsn(#[case] json_file_name: &str) {
        if let Err(err) = serde_json::from_str::<BrpPersons>(&read_json(json_file_name)) {
            assert!(err.to_string().starts_with("missing field"));
        } else {
            panic!("should fail deserializing JSON");
        }
    }

    #[rstest]
    #[case("huisletter", "20A")]
    #[case("huisletter-en-toevoeging", "1Abis")]
    #[case("huisnummertoevoeging", "38BIS1")]
    #[case("huisnummeraanduiding", "tegenover 38")]
    #[case("huisletter-en-aanduiding", "bij 38c")]
    #[case("huisletter-en-aanduiding-en-toevoeging-illegal-combination", "bij 38c")]
    fn should_handle_house_number(#[case] json_file_name: &str, #[case] expected_house_number: &str) {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json(json_file_name)).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert_eq!(expected_house_number, brp_person.residence.address.locator_designator());
    }

    #[test]
    fn should_convert_brp_person_to_issuable_vec() {
        let mut brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let issuable = brp_persons.persons.remove(0).into_issuable();

        assert_eq!(2, issuable.as_ref().len());

        let pid_card = &issuable.as_ref()[0];
        let address_card = &issuable.as_ref()[1];

        assert_eq!(
            json!([
                PID_ATTESTATION_TYPE,
                {
                    "family_name": "Jansen",
                    "given_name": "Frouke",
                    "birthdate": "2000-03-24",
                    "age_over_18": true,
                    "bsn": "999991772",
                    "nationalities": [
                        "Nederlandse",
                        "Belgische"
                    ]
                },
            ]),
            serde_json::to_value(pid_card).unwrap()
        );

        assert_eq!(
            json!([
                ADDRESS_ATTESTATION_TYPE,
                {
                    "address": {
                        "street_address": "Van Wijngaerdenstraat",
                        "house_number": "1",
                        "postal_code": "2596TW",
                        "locality": "Toetsoog",
                        "country": "Nederland",
                    },
                },
            ]),
            serde_json::to_value(address_card).unwrap()
        );
    }
}
