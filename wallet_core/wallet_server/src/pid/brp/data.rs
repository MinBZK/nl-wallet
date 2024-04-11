use std::ops::Add;

use chrono::{Days, Utc};
use ciborium::Value;
use indexmap::IndexMap;
use serde::Deserialize;

use nl_wallet_mdoc::{unsigned, unsigned::UnsignedMdoc, Tdate};

use crate::pid::constants::*;

#[derive(Debug, thiserror::Error)]
pub enum BrpDataError {
    #[error("there should at least be one nationality")]
    MissingNationality,
}

#[derive(Deserialize)]
pub struct BrpPersons {
    #[serde(rename = "personen")]
    pub persons: Vec<BrpPerson>,
}

#[derive(Deserialize)]
pub struct BrpPerson {
    #[serde(rename = "burgerservicenummer")]
    bsn: String,

    #[serde(rename = "geslacht")]
    gender: BrpGender,

    #[serde(rename = "naam")]
    name: BrpName,

    #[serde(rename = "geboorte")]
    birth: BrpBirth,

    #[serde(rename = "leeftijd")]
    age: u8,

    #[serde(rename = "verblijfplaats")]
    residence: BrpResidence,

    #[serde(rename = "nationaliteiten")]
    nationalities: Vec<BrpNationality>,
}

impl BrpPerson {
    fn is_over_18(&self) -> bool {
        self.age >= 18
    }

    fn nationalities_as_string(&self) -> Result<String, BrpDataError> {
        // Filter out optional nationalities that could not be deserialized, e.g. NationaliteitOnbekend.
        let nationatities = self
            .nationalities
            .iter()
            .filter_map(|nationality| nationality.nationality.clone().map(|n| n.name))
            .collect::<Vec<_>>();

        if nationatities.is_empty() {
            return Err(BrpDataError::MissingNationality);
        }

        Ok(nationatities.join(", "))
    }
}

impl TryFrom<&BrpPerson> for Vec<UnsignedMdoc> {
    type Error = BrpDataError;

    fn try_from(value: &BrpPerson) -> Result<Self, Self::Error> {
        let mdocs = vec![
            UnsignedMdoc {
                doc_type: String::from(MOCK_PID_DOCTYPE),
                copy_count: 2.try_into().unwrap(),
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(
                    String::from(MOCK_PID_DOCTYPE),
                    vec![
                        unsigned::Entry {
                            name: String::from(PID_BSN),
                            value: ciborium::Value::Text(value.bsn.clone()),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_FAMILY_NAME),
                            value: ciborium::Value::Text(value.name.family_name.clone()),
                        }
                        .into(),
                        value.name.family_name_prefix.clone().map(|prefix| unsigned::Entry {
                            name: String::from(PID_FAMILY_NAME_PREFIX),
                            value: ciborium::Value::Text(prefix),
                        }),
                        value.name.given_names.clone().map(|names| unsigned::Entry {
                            name: String::from(PID_GIVEN_NAME),
                            value: ciborium::Value::Text(names),
                        }),
                        unsigned::Entry {
                            name: String::from(PID_BIRTH_DATE),
                            value: ciborium::Value::Text(value.birth.date.date.format("%Y-%m-%d").to_string()),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_BIRTH_COUNTRY),
                            value: ciborium::Value::Text(value.birth.country.name.clone()),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_BIRTH_CITY),
                            value: ciborium::Value::Text(value.birth.place.name.clone()),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_AGE_OVER_18),
                            value: ciborium::Value::Bool(value.is_over_18()),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_GENDER),
                            value: value.gender.code.clone().into(),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_NATIONALITY),
                            value: ciborium::Value::Text(value.nationalities_as_string()?),
                        }
                        .into(),
                    ]
                    .into_iter()
                    .flatten()
                    .collect(),
                )]),
            },
            UnsignedMdoc {
                doc_type: String::from(MOCK_ADDRESS_DOCTYPE),
                copy_count: 2.try_into().unwrap(),
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(
                    String::from(MOCK_ADDRESS_DOCTYPE),
                    vec![
                        unsigned::Entry {
                            name: String::from(PID_RESIDENT_COUNTRY),
                            value: ciborium::Value::Text(value.residence.address.country.name.clone()),
                        }
                        .into(),
                        value.residence.address.street().map(|street| unsigned::Entry {
                            name: String::from(PID_RESIDENT_STREET),
                            value: ciborium::Value::Text(street),
                        }),
                        unsigned::Entry {
                            name: String::from(PID_RESIDENT_POSTAL_CODE),
                            value: ciborium::Value::Text(value.residence.address.postal_code.clone()),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_RESIDENT_HOUSE_NUMBER),
                            value: ciborium::Value::Text(value.residence.address.house_number.to_string()),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_RESIDENT_CITY),
                            value: ciborium::Value::Text(value.residence.address.city.clone()),
                        }
                        .into(),
                    ]
                    .into_iter()
                    .flatten()
                    .collect(),
                )]),
            },
        ];

        Ok(mdocs)
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

#[derive(Deserialize)]
pub struct BrpName {
    #[serde(rename = "voornamen")]
    given_names: Option<String>,

    #[serde(rename = "voorvoegsel")]
    family_name_prefix: Option<String>,

    #[serde(rename = "geslachtsnaam")]
    family_name: String,
}

#[derive(Deserialize)]
pub struct BrpBirth {
    #[serde(rename = "datum")]
    date: BrpBirthDate,

    #[serde(rename = "land")]
    country: BrpBirthCountry,

    #[serde(rename = "plaats")]
    place: BrpBirthPlace,
}

#[derive(Deserialize)]
pub struct BrpBirthDate {
    #[serde(rename = "datum")]
    date: chrono::NaiveDate,
}

#[derive(Deserialize)]
pub struct BrpBirthCountry {
    #[serde(rename = "omschrijving")]
    name: String,
}

#[derive(Deserialize)]
pub struct BrpBirthPlace {
    #[serde(rename = "omschrijving")]
    name: String,
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

    #[serde(rename = "postcode")]
    postal_code: String,

    #[serde(rename = "woonplaats")]
    city: String,

    #[serde(rename = "land", default)]
    country: BrpCountry,
}

impl BrpAddress {
    fn street(&self) -> Option<String> {
        self.official_street_name.clone().or(self.short_street_name.clone())
    }
}

#[derive(Deserialize)]
pub struct BrpCountry {
    #[serde(rename = "omschrijving")]
    name: String,
}

impl Default for BrpCountry {
    fn default() -> Self {
        Self {
            name: String::from("Nederland"),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct BrpNationality {
    #[serde(rename = "nationaliteit")]
    nationality: Option<BrpNationalityName>,
}

#[derive(Deserialize, Clone)]
pub struct BrpNationalityName {
    #[serde(rename = "omschrijving")]
    name: String,
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf};

    use indexmap::IndexMap;
    use rstest::rstest;

    use nl_wallet_mdoc::{
        unsigned::{Entry, UnsignedMdoc},
        NameSpace,
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

    fn readable_attrs(attrs: &IndexMap<NameSpace, Vec<Entry>>) -> Vec<(String, String)> {
        attrs
            .iter()
            .flat_map(|(_ns, entries)| {
                entries
                    .iter()
                    .map(|entry| (entry.name.clone(), String::from(entry.value.as_text().unwrap_or(""))))
            })
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

    #[test]
    fn should_return_multiple_nationalities() {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json("multiple-nationalities")).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert_eq!("Belgische, Nederlandse", brp_person.nationalities_as_string().unwrap());
    }

    #[rstest]
    #[case("empty-nationalities")]
    #[case("unknown-nationalities")]
    fn should_err_for_missing_nationality(#[case] json_file_name: &str) {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json(json_file_name)).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        brp_person
            .nationalities_as_string()
            .expect_err("should error when nationality is missing");
    }

    #[test]
    fn should_convert_brp_person_to_mdoc() {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let unsigned_mdoc: Vec<UnsignedMdoc> = brp_persons.persons.first().unwrap().try_into().unwrap();

        assert_eq!(2, unsigned_mdoc.len());

        let pid_card = &unsigned_mdoc[0];
        let address_card = &unsigned_mdoc[1];

        assert_eq!(
            vec![
                ("bsn", "999991772"),
                ("family_name", "Jansen"),
                ("given_name", "Frouke"),
                ("birth_date", "2000-03-24"),
                ("birth_country", "België"),
                ("birth_city", "Luik"),
                ("age_over_18", ""),
                ("gender", ""),
                ("nationality", "Nederlandse"),
            ],
            readable_attrs(&pid_card.attributes)
                .iter()
                .map(|(a, b)| (a.as_str(), b.as_str()))
                .collect::<Vec<_>>()
        );

        assert_eq!(
            vec![
                ("resident_country", "Nederland"),
                ("resident_street", "Van Wijngaerdenstraat"),
                ("resident_postal_code", "2596TW"),
                ("resident_house_number", "1"),
                ("resident_city", "Toetsoog"),
            ],
            readable_attrs(&address_card.attributes)
                .iter()
                .map(|(a, b)| (a.as_str(), b.as_str()))
                .collect::<Vec<_>>()
        );
    }
}
