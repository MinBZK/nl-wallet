use std::collections::HashMap;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::Attributes;
use chrono::NaiveDate;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde_valid::Validate;

use crate::pid::rulebook::constants::PID_BASE_MDOC_NAMESPACE;
use crate::pid::rulebook::constants::PID_BIRTH_PLACE_GROUP;
use crate::pid::rulebook::constants::PID_BSN;
use crate::pid::rulebook::constants::PID_FAMILY_NAME;
use crate::pid::rulebook::constants::PID_GIVEN_NAME;
use crate::pid::rulebook::constants::PID_MDOC_BIRTH_DATE;
use crate::pid::rulebook::constants::PID_MDOC_FAMILY_NAME_BIRTH;
use crate::pid::rulebook::constants::PID_MDOC_NATIONALITY;
use crate::pid::rulebook::constants::PID_MDOC_RESIDENT_CITY;
use crate::pid::rulebook::constants::PID_MDOC_RESIDENT_COUNTRY;
use crate::pid::rulebook::constants::PID_MDOC_RESIDENT_POSTAL_CODE;
use crate::pid::rulebook::constants::PID_MDOC_RESIDENT_STATE;
use crate::pid::rulebook::constants::PID_MDOC_RESIDENT_STREET;
use crate::pid::rulebook::constants::PID_NL_MDOC_NAMESPACE;
use crate::pid::rulebook::constants::PID_RECOVERY_CODE;
use crate::pid::rulebook::constants::PID_SD_JWT_ADDRESS_GROUP;
use crate::pid::rulebook::constants::PID_SD_JWT_BIRTH_DATE;
use crate::pid::rulebook::constants::PID_SD_JWT_COUNTRY;
use crate::pid::rulebook::constants::PID_SD_JWT_FAMILY_NAME_BIRTH;
use crate::pid::rulebook::constants::PID_SD_JWT_HOUSE_NUMBER;
use crate::pid::rulebook::constants::PID_SD_JWT_LOCALITY;
use crate::pid::rulebook::constants::PID_SD_JWT_NATIONALITIES;
use crate::pid::rulebook::constants::PID_SD_JWT_POSTAL_CODE;
use crate::pid::rulebook::constants::PID_SD_JWT_REGION;
use crate::pid::rulebook::constants::PID_SD_JWT_STREET_ADDRESS;
use crate::pid::rulebook::constants::PID_SEX;
use crate::pid::rulebook::country_code::RulebookCountryCode;
use crate::pid::rulebook::country_code::RulebookNationalities;
use crate::pid::rulebook::issuer_metadata::PidIssuerMetadata;
use crate::pid::rulebook::sex::RulebookSex;

#[derive(Deserialize, derive_more::AsRef)]
pub struct Persons(HashMap<String, Person>);

#[derive(Debug, thiserror::Error)]
#[error("duplicate bsn: {0}")]
pub struct DuplicateBsnError(String);

impl From<Person> for Persons {
    fn from(person: Person) -> Self {
        Self(HashMap::from([(person.bsn.clone(), person)]))
    }
}

impl TryFrom<Vec<Person>> for Persons {
    type Error = DuplicateBsnError;

    fn try_from(persons: Vec<Person>) -> Result<Self, Self::Error> {
        let mut map = HashMap::with_capacity(persons.len());
        for person in persons {
            if let Some(person) = map.insert(person.bsn.clone(), person) {
                return Err(DuplicateBsnError(person.bsn));
            }
        }
        Ok(Self(map))
    }
}

/// Person attributes are modeled after PID rulebook
#[derive(Clone, Deserialize)]
pub struct Person {
    pub bsn: String,
    pub family_name: String,
    pub family_name_birth: Option<String>,
    pub given_name: String,
    pub birth_date: NaiveDate,
    pub birth_place: BirthPlace,
    pub nationalities: RulebookNationalities,
    pub address: Option<Address>,
    pub sex: Option<RulebookSex>,
}

#[derive(Clone, Deserialize, Validate)]
#[validate(custom = validate_birth_place)]
pub struct BirthPlace {
    pub country: Option<RulebookCountryCode>,
    pub region: Option<String>,
    pub locality: Option<String>,
}

fn validate_birth_place(birth_place: &BirthPlace) -> Result<(), serde_valid::validation::Error> {
    if birth_place.country.is_some() || birth_place.region.is_some() || birth_place.locality.is_some() {
        Ok(())
    } else {
        Err(serde_valid::validation::Error::Custom(
            "birth place should have at least a country, region or locality".into(),
        ))
    }
}

#[derive(Clone, Deserialize)]
pub struct Address {
    pub country: Option<RulebookCountryCode>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub street: Option<String>,
    pub house_number: Option<String>,
}

impl Person {
    pub fn into_sd_jwt_attributes(self, recovery_code: String, issuer_metadata: PidIssuerMetadata) -> Attributes {
        let mut attributes = IndexMap::from([
            (PID_RECOVERY_CODE.to_owned(), Attribute::Text(recovery_code)),
            (PID_BSN.to_owned(), Attribute::Text(self.bsn)),
            (PID_FAMILY_NAME.to_owned(), Attribute::Text(self.family_name)),
            (PID_GIVEN_NAME.to_owned(), Attribute::Text(self.given_name)),
            (
                PID_SD_JWT_BIRTH_DATE.to_owned(),
                Attribute::Text(self.birth_date.format("%Y-%m-%d").to_string()),
            ),
            (PID_BIRTH_PLACE_GROUP.to_owned(), self.birth_place.into_attribute()),
            (PID_SD_JWT_NATIONALITIES.to_owned(), self.nationalities.into()),
        ]);
        attributes.extend(
            [
                self.address
                    .and_then(Address::into_sd_jwt_attribute)
                    .map(|attr| (PID_SD_JWT_ADDRESS_GROUP.to_owned(), attr)),
                self.family_name_birth
                    .map(|name| (PID_SD_JWT_FAMILY_NAME_BIRTH.to_owned(), Attribute::Text(name))),
                self.sex.map(|sex| (PID_SEX.to_owned(), sex.into())),
            ]
            .into_iter()
            .flatten(),
        );
        attributes.extend(issuer_metadata.into_attributes());

        Attributes::from(attributes)
    }

    pub fn into_mdoc_attributes(self, recovery_code: String, issuer_metadata: PidIssuerMetadata) -> Attributes {
        let mut attributes = IndexMap::from([
            (PID_FAMILY_NAME.to_owned(), Attribute::Text(self.family_name)),
            (PID_GIVEN_NAME.to_owned(), Attribute::Text(self.given_name)),
            (PID_MDOC_BIRTH_DATE.to_owned(), Attribute::Date(self.birth_date)),
            (PID_BIRTH_PLACE_GROUP.to_owned(), self.birth_place.into_attribute()),
            (PID_MDOC_NATIONALITY.to_owned(), self.nationalities.into()),
        ]);
        if let Some(address) = self.address {
            attributes.extend(address.into_mdoc_address_attributes());
        }
        attributes.extend(
            [
                self.family_name_birth
                    .map(|name| (PID_MDOC_FAMILY_NAME_BIRTH.to_owned(), Attribute::Text(name))),
                self.sex.map(|sex| (PID_SEX.to_owned(), sex.into())),
            ]
            .into_iter()
            .flatten(),
        );
        attributes.extend(issuer_metadata.into_attributes());

        let nl_attributes = IndexMap::from([
            (PID_BSN.to_owned(), Attribute::Text(self.bsn)),
            (PID_RECOVERY_CODE.to_owned(), Attribute::Text(recovery_code)),
        ]);

        Attributes::from(IndexMap::from([
            (PID_BASE_MDOC_NAMESPACE.to_owned(), Attribute::Object(attributes)),
            (PID_NL_MDOC_NAMESPACE.to_owned(), Attribute::Object(nl_attributes)),
        ]))
    }
}

impl Address {
    pub fn into_sd_jwt_attribute(self) -> Option<Attribute> {
        let attributes = [
            self.street
                .map(|street| (PID_SD_JWT_STREET_ADDRESS.to_owned(), Attribute::Text(street))),
            self.house_number
                .map(|house_number| (PID_SD_JWT_HOUSE_NUMBER.to_owned(), Attribute::Text(house_number))),
            self.postal_code
                .map(|postal_code| (PID_SD_JWT_POSTAL_CODE.to_owned(), Attribute::Text(postal_code))),
            self.city
                .map(|city| (PID_SD_JWT_LOCALITY.to_owned(), Attribute::Text(city))),
            self.state
                .map(|state| (PID_SD_JWT_REGION.to_owned(), Attribute::Text(state))),
            self.country
                .map(|country| (PID_SD_JWT_COUNTRY.to_owned(), country.into())),
        ]
        .into_iter()
        .flatten()
        .collect::<IndexMap<_, _>>();

        (!attributes.is_empty()).then(|| Attribute::Object(attributes))
    }

    /// mdoc has no nested `address` and uses dedicated attribute identifiers (`resident_*`)
    pub fn into_mdoc_address_attributes(self) -> impl Iterator<Item = (String, Attribute)> {
        [
            self.mdoc_street()
                .map(|street| (PID_MDOC_RESIDENT_STREET.to_owned(), Attribute::Text(street))),
            self.postal_code
                .map(|postal_code| (PID_MDOC_RESIDENT_POSTAL_CODE.to_owned(), Attribute::Text(postal_code))),
            self.city
                .map(|city| (PID_MDOC_RESIDENT_CITY.to_owned(), Attribute::Text(city))),
            self.state
                .map(|state| (PID_MDOC_RESIDENT_STATE.to_owned(), Attribute::Text(state))),
            self.country
                .map(|country| (PID_MDOC_RESIDENT_COUNTRY.to_owned(), country.into())),
        ]
        .into_iter()
        .flatten()
    }

    pub fn mdoc_street(&self) -> Option<String> {
        let street = [&self.street, &self.house_number].into_iter().flatten().join(" ");
        (!street.is_empty()).then_some(street)
    }
}

impl BirthPlace {
    /// Builds the `place_of_birth` attribute
    ///
    /// ```cddl
    /// cddl place_of_birth = {
    ///     ? "country": tstr  ; a single alpha-2 country code as specified in [ISO 3166-1]
    ///     ? "region": tstr   ; the name of a state, province, district, or local area
    ///     ? "locality": tstr ; the name of a municipality, city, town, or village
    /// }
    /// ```
    fn into_attribute(self) -> Attribute {
        let group = [
            self.country
                .map(|country| (PID_SD_JWT_COUNTRY.to_owned(), Attribute::from(country))),
            self.region
                .map(|region| (PID_SD_JWT_REGION.to_owned(), Attribute::Text(region))),
            self.locality
                .map(|locality| (PID_SD_JWT_LOCALITY.to_owned(), Attribute::Text(locality))),
        ]
        .into_iter()
        .flatten()
        .collect::<IndexMap<_, _>>();

        // place_of_birth SHALL contain at least one of the following key-value pairs: "country",
        // "region", or "locality". This is checked by the `validate` attribute.
        assert!(
            !group.is_empty(),
            "birth place should have at least a country, region or locality"
        );
        Attribute::Object(group)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::pid::mock::MOCK_PERSON;
    use crate::pid::rulebook::country_code::IsoCountryCode;

    #[test]
    fn should_convert_person_to_sd_jwt_issuable_vec() {
        let issuer_metadata = PidIssuerMetadata {
            issuing_country: IsoCountryCode::Netherlands,
            issuing_authority: "Test Issuing Authority".to_owned(),
        };
        let pid_card = MOCK_PERSON
            .clone()
            .into_sd_jwt_attributes("recovery_code_123".into(), issuer_metadata);

        assert_eq!(
            json!({
                "bsn": {
                    "type": "text",
                    "value": "999991772"
                },
                "family_name": {
                    "type": "text",
                    "value": "Jansen"
                },
                "given_name": {
                    "type": "text",
                    "value": "Frouke"
                },
                "birthdate": {
                    "type": "text",
                    "value": "2000-03-24"
                },
                "place_of_birth": {
                    "type": "object",
                    "value": {
                        "country": {
                            "type": "text",
                            "value": "NL"
                        }
                    }
                },
                "nationalities": {
                    "type": "array",
                    "value": [
                        {
                            "type": "text",
                            "value": "NL"
                        },
                        {
                            "type": "text",
                            "value": "BE"
                        }
                    ]
                },
                "birth_family_name": {
                    "type": "text",
                    "value": "Jansen"
                },
                "sex": {
                    "type": "number",
                    "value": 2
                },
                "issuing_country": {
                    "type": "text",
                    "value": "NL"
                },
                "issuing_authority": {
                    "type": "text",
                    "value": "Test Issuing Authority"
                },
                "recovery_code": {
                    "type": "text",
                    "value": "recovery_code_123"
                },
                "address": {
                    "type": "object",
                    "value": {
                        "street_address": {
                            "type": "text",
                            "value": "Van Wijngaerdenstraat"
                        },
                        "house_number": {
                            "type": "text",
                            "value": "1"
                        },
                        "postal_code": {
                            "type": "text",
                            "value": "2596TW"
                        },
                        "locality": {
                            "type": "text",
                            "value": "Toetsoog"
                        },
                        "country": {
                            "type": "text",
                            "value": "NL"
                        }
                    }
                }
            }),
            serde_json::to_value(pid_card).unwrap(),
        );
    }

    #[test]
    fn should_convert_person_to_mdoc_issuable_vec() {
        let issuer_metadata = PidIssuerMetadata {
            issuing_country: IsoCountryCode::Netherlands,
            issuing_authority: "Test Issuing Authority".to_owned(),
        };
        let pid_card = MOCK_PERSON
            .clone()
            .into_mdoc_attributes("recovery_code_123".into(), issuer_metadata);

        assert_eq!(
            json!({
                "eu.europa.ec.eudi.pid.1": {
                    "type": "object",
                    "value": {
                        "family_name": {
                            "type": "text",
                            "value": "Jansen"
                        },
                        "given_name": {
                            "type": "text",
                            "value": "Frouke"
                        },
                        "birth_date": {
                            "type": "date",
                            "value": "2000-03-24"
                        },
                        "place_of_birth": {
                            "type": "object",
                            "value": {
                                "country": {
                                    "type": "text",
                                    "value": "NL"
                                }
                            }
                        },
                        "nationality": {
                            "type": "array",
                            "value": [
                                {
                                    "type": "text",
                                    "value": "NL"
                                },
                                {
                                    "type": "text",
                                    "value": "BE"
                                }
                            ]
                        },
                        "family_name_birth": {
                            "type": "text",
                            "value": "Jansen"
                        },
                        "sex": {
                            "type": "number",
                            "value": 2
                        },
                        "issuing_country": {
                            "type": "text",
                            "value": "NL"
                        },
                        "issuing_authority": {
                            "type": "text",
                            "value": "Test Issuing Authority"
                        },
                        "resident_street": {
                            "type": "text",
                            "value": "Van Wijngaerdenstraat 1"
                        },
                        "resident_postal_code": {
                            "type": "text",
                            "value": "2596TW"
                        },
                        "resident_city": {
                            "type": "text",
                            "value": "Toetsoog"
                        },
                        "resident_country": {
                            "type": "text",
                            "value": "NL"
                        }
                    }
                },
                "eu.europa.ec.eudi.pid.nl.1": {
                    "type": "object",
                    "value": {
                        "bsn": {
                            "type": "text",
                            "value": "999991772"
                        },
                        "recovery_code": {
                            "type": "text",
                            "value": "recovery_code_123"
                        }
                    }
                }
            }),
            serde_json::to_value(pid_card).unwrap(),
        );
    }
}
