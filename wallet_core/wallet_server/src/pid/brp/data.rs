use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::Deserialize;

use openid4vc::attributes::Attribute;
use openid4vc::attributes::AttributeValue;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuable_document::IssuableDocuments;

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

    #[serde(rename = "geslacht")]
    gender: Option<BrpGender>,

    #[serde(rename = "naam")]
    name: BrpName,

    #[serde(rename = "geboorte")]
    birth: BrpBirth,

    #[serde(rename = "leeftijd")]
    age: u8,

    #[serde(rename = "verblijfplaats")]
    residence: BrpResidence,
}

impl BrpPerson {
    fn is_over_18(&self) -> bool {
        self.age >= 18
    }
}

impl From<BrpPerson> for IssuableDocuments {
    fn from(value: BrpPerson) -> Self {
        let given_names = value.name.given_names.clone();
        let is_over_18 = value.is_over_18();
        let family_name = value.name.into_name_with_prefix();
        let birth_country = value.birth.country;
        let birth_place = value.birth.place;
        let street = value.residence.address.street().map(String::from);
        let house_number = value.residence.address.locator_designator();

        vec![
            IssuableDocument::try_new(
                String::from(MOCK_PID_DOCTYPE),
                IndexMap::from_iter(
                    vec![
                        Some((
                            String::from(PID_BSN),
                            Attribute::Single(AttributeValue::Text(value.bsn)),
                        )),
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
                                value.birth.date.date.format("%Y-%m-%d").to_string(),
                            )),
                        )),
                        birth_country.map(|country| {
                            (
                                String::from(PID_BIRTH_COUNTRY),
                                Attribute::Single(AttributeValue::Text(country.description)),
                            )
                        }),
                        birth_place.map(|place| {
                            (
                                String::from(PID_BIRTH_CITY),
                                Attribute::Single(AttributeValue::Text(place.description)),
                            )
                        }),
                        Some((
                            String::from(PID_AGE_OVER_18),
                            Attribute::Single(AttributeValue::Bool(is_over_18)),
                        )),
                        value
                            .gender
                            .map(|gender| (String::from(PID_GENDER), gender.code.into())),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(String, Attribute)>>(),
                ),
            )
            .unwrap(),
            IssuableDocument::try_new(
                String::from(MOCK_ADDRESS_DOCTYPE),
                IndexMap::from_iter(
                    vec![
                        Some((
                            String::from(PID_RESIDENT_COUNTRY),
                            Attribute::Single(AttributeValue::Text(value.residence.address.country.description)),
                        )),
                        street.map(|street| {
                            (
                                String::from(PID_RESIDENT_STREET),
                                Attribute::Single(AttributeValue::Text(street)),
                            )
                        }),
                        Some((
                            String::from(PID_RESIDENT_POSTAL_CODE),
                            Attribute::Single(AttributeValue::Text(value.residence.address.postal_code)),
                        )),
                        Some((
                            String::from(PID_RESIDENT_HOUSE_NUMBER),
                            Attribute::Single(AttributeValue::Text(house_number)),
                        )),
                        Some((
                            String::from(PID_RESIDENT_CITY),
                            Attribute::Single(AttributeValue::Text(value.residence.address.city)),
                        )),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(String, Attribute)>>(),
                ),
            )
            .unwrap(),
        ]
        .try_into()
        .unwrap()
    }
}

#[derive(Deserialize)]
pub struct BrpGender {
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
        Attribute::Single(AttributeValue::Number(value))
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

    #[serde(rename = "land")]
    country: Option<BrpDescription>,

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
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    use rstest::rstest;

    use serde_json::json;

    use openid4vc::issuable_document::IssuableDocuments;

    use crate::pid::brp::data::BrpPersons;

    fn read_json(name: &str) -> String {
        fs::read_to_string(
            env::var("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .unwrap()
                .join(format!("resources/test/haal-centraal-examples/{}.json", name)),
        )
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
    fn should_convert_brp_person_to_issuable_attributes() {
        let mut brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let issuable: IssuableDocuments = brp_persons.persons.remove(0).into();

        assert_eq!(2, issuable.as_ref().len());

        let pid_card = &issuable.as_ref()[0];
        let address_card = &issuable.as_ref()[1];

        assert_eq!(
            json!({
                "attestation_type": "com.example.pid",
                "attributes": {
                    "bsn": "999991772",
                    "family_name": "Jansen",
                    "given_name": "Frouke",
                    "birth_date": "2000-03-24",
                    "age_over_18": true,
                    "gender": 2,
                },
            }),
            serde_json::to_value(pid_card).unwrap()
        );

        assert_eq!(
            json!({
                "attestation_type": "com.example.address",
                "attributes": {
                    "resident_country": "Nederland",
                    "resident_street": "Van Wijngaerdenstraat",
                    "resident_postal_code": "2596TW",
                    "resident_house_number": "1",
                    "resident_city": "Toetsoog",
                },
            }),
            serde_json::to_value(address_card).unwrap()
        );
    }
}
