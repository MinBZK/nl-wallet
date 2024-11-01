use std::{num::NonZeroU8, ops::Add};

use chrono::{Days, NaiveDate, Utc};
use ciborium::Value;
use indexmap::IndexMap;
use serde::Deserialize;

use nl_wallet_mdoc::{unsigned, unsigned::UnsignedMdoc, Tdate};

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

impl From<BrpPerson> for Vec<UnsignedMdoc> {
    fn from(value: BrpPerson) -> Self {
        let given_names = value.name.given_names.clone();
        let is_over_18 = value.is_over_18();
        let family_name = value.name.into_name_with_prefix();
        let birth_country = value.birth.country;
        let birth_place = value.birth.place;
        let street = value.residence.address.street().map(String::from);
        let house_number = value.residence.address.locator_designator();

        vec![
            UnsignedMdoc {
                doc_type: String::from(MOCK_PID_DOCTYPE),
                copy_count: NonZeroU8::new(2).unwrap(),
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(
                    String::from(MOCK_PID_DOCTYPE),
                    vec![
                        unsigned::Entry {
                            name: String::from(PID_BSN),
                            value: ciborium::Value::Text(value.bsn),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_FAMILY_NAME),
                            value: ciborium::Value::Text(family_name),
                        }
                        .into(),
                        given_names.map(|names| unsigned::Entry {
                            name: String::from(PID_GIVEN_NAME),
                            value: ciborium::Value::Text(names),
                        }),
                        unsigned::Entry {
                            name: String::from(PID_BIRTH_DATE),
                            value: ciborium::Value::Text(value.birth.date.date.format("%Y-%m-%d").to_string()),
                        }
                        .into(),
                        birth_country.map(|country| unsigned::Entry {
                            name: String::from(PID_BIRTH_COUNTRY),
                            value: ciborium::Value::Text(country.description),
                        }),
                        birth_place.map(|place| unsigned::Entry {
                            name: String::from(PID_BIRTH_CITY),
                            value: ciborium::Value::Text(place.description),
                        }),
                        unsigned::Entry {
                            name: String::from(PID_AGE_OVER_18),
                            value: ciborium::Value::Bool(is_over_18),
                        }
                        .into(),
                        value.gender.map(|gender| unsigned::Entry {
                            name: String::from(PID_GENDER),
                            value: gender.code.into(),
                        }),
                    ]
                    .into_iter()
                    .flatten()
                    .collect(),
                )])
                .try_into()
                .unwrap(),
            },
            UnsignedMdoc {
                doc_type: String::from(MOCK_ADDRESS_DOCTYPE),
                copy_count: NonZeroU8::new(2).unwrap(),
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(
                    String::from(MOCK_ADDRESS_DOCTYPE),
                    vec![
                        unsigned::Entry {
                            name: String::from(PID_RESIDENT_COUNTRY),
                            value: ciborium::Value::Text(value.residence.address.country.description),
                        }
                        .into(),
                        street.map(|street| unsigned::Entry {
                            name: String::from(PID_RESIDENT_STREET),
                            value: ciborium::Value::Text(street),
                        }),
                        unsigned::Entry {
                            name: String::from(PID_RESIDENT_POSTAL_CODE),
                            value: ciborium::Value::Text(value.residence.address.postal_code),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_RESIDENT_HOUSE_NUMBER),
                            value: ciborium::Value::Text(house_number),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_RESIDENT_CITY),
                            value: ciborium::Value::Text(value.residence.address.city),
                        }
                        .into(),
                    ]
                    .into_iter()
                    .flatten()
                    .collect(),
                )])
                .try_into()
                .unwrap(),
            },
        ]
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

impl From<BrpGenderCode> for ciborium::Value {
    fn from(value: BrpGenderCode) -> Value {
        let value = match value {
            BrpGenderCode::O => 0,
            BrpGenderCode::M => 1,
            BrpGenderCode::V => 2,
        };
        Value::Integer(value.into())
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
    use std::{env, fs, path::PathBuf};

    use indexmap::IndexMap;
    use rstest::rstest;

    use nl_wallet_mdoc::{
        unsigned::{Entry, UnsignedMdoc},
        DataElementValue, NameSpace,
    };

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

    fn readable_attrs(attrs: &IndexMap<NameSpace, Vec<Entry>>) -> Vec<(&str, DataElementValue)> {
        attrs
            .iter()
            .flat_map(|(_ns, entries)| entries.iter().map(|entry| (entry.name.as_str(), entry.value.clone())))
            .collect::<Vec<_>>()
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
    fn should_convert_brp_person_to_mdoc() {
        let mut brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let unsigned_mdoc: Vec<UnsignedMdoc> = brp_persons.persons.remove(0).into();

        assert_eq!(2, unsigned_mdoc.len());

        let pid_card = &unsigned_mdoc[0];
        let address_card = &unsigned_mdoc[1];

        assert_eq!(
            vec![
                ("bsn", "999991772".into()),
                ("family_name", "Jansen".into()),
                ("given_name", "Frouke".into()),
                ("birth_date", "2000-03-24".into()),
                ("age_over_18", true.into()),
                ("gender", 2.into()),
            ],
            readable_attrs(pid_card.attributes.as_ref())
        );

        assert_eq!(
            vec![
                ("resident_country", "Nederland".into()),
                ("resident_street", "Van Wijngaerdenstraat".into()),
                ("resident_postal_code", "2596TW".into()),
                ("resident_house_number", "1".into()),
                ("resident_city", "Toetsoog".into()),
            ],
            readable_attrs(address_card.attributes.as_ref())
        );
    }
}
