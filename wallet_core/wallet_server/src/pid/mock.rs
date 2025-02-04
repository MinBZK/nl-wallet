/// Mock implementations of the two traits abstracting other components
use std::collections::HashMap;

use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;

use openid4vc::attributes::Attribute;
use openid4vc::attributes::AttributeValue;
use openid4vc::attributes::IssuableDocument;
use openid4vc::attributes::IssuableDocuments;
use sd_jwt::metadata::TypeMetadata;

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
        AttributeValue::Number(value.into())
    }
}

#[derive(Default, Deserialize, Clone)]
pub struct PersonAttributes {
    bsn: String,
    family_name: String,
    given_name: String,
    birth_date: NaiveDate,
    age_over_18: bool,
}

impl From<PersonAttributes> for IssuableDocument {
    fn from(value: PersonAttributes) -> Self {
        Self::try_new(
            MOCK_PID_DOCTYPE.to_string(),
            vec![
                (PID_BSN.to_string(), Attribute::Single(AttributeValue::Text(value.bsn))),
                (
                    PID_FAMILY_NAME.to_string(),
                    Attribute::Single(AttributeValue::Text(value.family_name)),
                ),
                (
                    PID_GIVEN_NAME.to_string(),
                    Attribute::Single(AttributeValue::Text(value.given_name)),
                ),
                (
                    PID_BIRTH_DATE.to_string(),
                    Attribute::Single(AttributeValue::Text(value.birth_date.format("%Y-%m-%d").to_string())),
                ),
                (
                    PID_AGE_OVER_18.to_string(),
                    Attribute::Single(AttributeValue::Bool(value.age_over_18)),
                ),
            ]
            .into_iter()
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
                    birth_date: NaiveDate::parse_from_str("1997-05-10", "%Y-%m-%d").unwrap(),
                    age_over_18: true,
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

pub fn mock_pid_metadata() -> TypeMetadata {
    let json = json!(
        {
      "vct": "com.example.pid",
      "name": "NL Wallet PID credential",
      "description": "Working version of the NL Wallet PID credential",
      "display": [
        {
          "lang": "en-US",
          "name": "NL Wallet Personal Data",
          "description": "The Personal Data credential for the NL Wallet"
        },
        {
          "lang": "nl-NL",
          "name": "NL Wallet persoonsgegevens",
          "description": "De persoonsgegevensattestatie voor de NL Wallet"
        }
      ],
      "claims": [
        {
          "path": ["family_name"],
          "display": [
            {
              "lang": "nl-NL",
              "label": "Achternaam",
              "description": "Achternaam van de persoon, inclusief voorvoegsels"
            },
            {
              "lang": "en-US",
              "label": "Name",
              "description": "Family name of the person, including any prefixes"
            }
          ],
          "sd": "always"
        },
        {
          "path": ["given_name"],
          "display": [
            {
              "lang": "nl-NL",
              "label": "Voornaam",
              "description": "Voornaam van de persoon"
            },
            {
              "lang": "en-US",
              "label": "First name",
              "description": "First name of the person"
            }
          ],
          "sd": "always"
        },
        {
          "path": ["birth_date"],
          "display": [
            {
              "lang": "nl-NL",
              "label": "Geboortedatum",
              "description": "Geboortedatum van de persoon"
            },
            {
              "lang": "en-US",
              "label": "Birth date",
              "description": "Birth date of the person"
            }
          ],
          "sd": "always"
        },
        {
          "path": ["age_over_18"],
          "display": [
            {
              "lang": "nl-NL",
              "label": "18+",
              "description": "Of de persoon 18+ is"
            },
            {
              "lang": "en-US",
              "label": "Over 18",
              "description": "Whether the person is over 18"
            }
          ],
          "sd": "always"
        },
        {
          "path": ["bsn"],
          "display": [
            {
              "lang": "nl-NL",
              "label": "BSN",
              "description": "BSN van de persoon"
            },
            {
              "lang": "en-US",
              "label": "BSN",
              "description": "BSN of the person"
            }
          ],
          "sd": "always"
        }
      ],
      "schema": {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "NL Wallet PID VCT Schema",
        "description": "The JSON schema that defines the NL Wallet PID VCT",
        "type": "object",
        "properties": {
          "given_name": {
            "type": "string"
          },
          "family_name": {
            "type": "string"
          },
          "birthdate": {
            "type": "string",
            "format": "date"
          },
          "age_over_18": {
            "type": "boolean"
          },
          "bsn": {
            "type": "string"
          },
          "vct": {
            "type": "string"
          },
          "vct#integrity": {
            "type": "string"
          },
          "iss": {
            "type": "string"
          },
          "iat": {
            "type": "number"
          },
          "exp": {
            "type": "number"
          }
        },
        "required": ["vct", "iss", "iat", "exp"],
        "additionalProperties": false
      }
    });

    serde_json::from_value(json).unwrap()
}

pub fn mock_address_metadata() -> TypeMetadata {
    let json = json!({
      "vct": "com.example.address",
      "name": "NL Wallet address credential",
      "description": "Working version of the NL Wallet address credential",
      "display": [
        {
          "lang": "en-US",
          "name": "NL Wallet address",
          "description": "The address credential for the NL Wallet"
        },
        {
          "lang": "nl-NL",
          "name": "NL Wallet adres",
          "description": "De adresattestatie voor de NL Wallet"
        }
      ],
      "claims": [
        {
          "path": [
            "resident_street"
          ],
          "display": [
            {
              "lang": "nl-NL",
              "label": "Straatnaam",
              "description": "Straatnaam van het adres"
            },
            {
              "lang": "en-US",
              "label": "Street",
              "description": "Street of the address"
            }
          ],
          "sd": "always"
        },
        {
          "path": [
            "resident_house_number"
          ],
          "display": [
            {
              "lang": "nl-NL",
              "label": "Huisnummer",
              "description": "Huisnummer van het adres"
            },
            {
              "lang": "en-US",
              "label": "House number",
              "description": "House number of the address"
            }
          ],
          "sd": "always"
        },
        {
          "path": [
            "resident_postal_code"
          ],
          "display": [
            {
              "lang": "nl-NL",
              "label": "Postcode",
              "description": "Postcode van het adres"
            },
            {
              "lang": "en-US",
              "label": "Postal code",
              "description": "Postal code of the address"
            }
          ],
          "sd": "always"
        },
        {
          "path": [
            "resident_city"
          ],
          "display": [
            {
              "lang": "nl-NL",
              "label": "Stad",
              "description": "Stad van het adres"
            },
            {
              "lang": "en-US",
              "label": "City",
              "description": "City of the address"
            }
          ],
          "sd": "always"
        }
      ],
      "schema": {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "NL Wallet address VCT Schema",
        "description": "The JSON schema that defines the NL Wallet address VCT",
        "type": "object",
        "properties": {
          "resident_street": {
            "type": "string"
          },
          "resident_house_number": {
            "type": "string"
          },
          "resident_postal_code": {
            "type": "string"
          },
          "resident_city": {
            "type": "boolean"
          },
          "vct": {
            "type": "string"
          },
          "vct#integrity": {
            "type": "string"
          },
          "iss": {
            "type": "string"
          },
          "iat": {
            "type": "string"
          },
          "exp": {
            "type": "string"
          }
        },
        "required": [
          "vct",
          "iss",
          "iat",
          "exp"
        ],
        "additionalProperties": false
      }
    });

    serde_json::from_value(json).unwrap()
}
