use std::{num::NonZeroU8, ops::Add};

use chrono::{Days, NaiveDate, Utc};
use ciborium::Value;
use indexmap::IndexMap;
use serde::Deserialize;

use nl_wallet_mdoc::{unsigned, unsigned::UnsignedMdoc, Tdate};

use crate::pid::constants::*;

#[derive(Debug, thiserror::Error)]
pub enum BrpDataError {
    #[error("missing partner")]
    MissingPartner,
}

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
    gender: BrpGender,

    #[serde(rename = "naam")]
    name: BrpName,

    #[serde(rename = "geboorte")]
    birth: BrpBirth,

    #[serde(rename = "leeftijd")]
    age: u8,

    #[serde(rename = "verblijfplaats")]
    residence: BrpResidence,

    #[serde(rename = "partners", default)]
    partners: Vec<BrpPartner>,
}

impl BrpPerson {
    fn is_over_18(&self) -> bool {
        self.age >= 18
    }

    fn partner(&self) -> Option<&BrpPartner> {
        self.partners
            .iter()
            .find(|partner| partner.start.is_some() && partner.end.is_none())
    }

    fn has_spouse_or_partner(&self) -> bool {
        self.partner()
            .map(|partner| partner.kind != BrpMaritalStatus::Onbekend)
            .unwrap_or(false)
    }
}

impl TryFrom<BrpPerson> for Vec<UnsignedMdoc> {
    type Error = BrpDataError;

    fn try_from(value: BrpPerson) -> Result<Self, Self::Error> {
        let family_name = value
            .name
            .clone()
            .info_family_name(value.partner().cloned().map(|p| p.name))?;
        let given_names = value.name.given_names.clone();
        let is_over_18 = value.is_over_18();
        let has_spouse_or_partner = value.has_spouse_or_partner();
        let birth_country = value.birth.country;
        let birth_place = value.birth.place;
        let street = value.residence.address.street();

        let mdocs = vec![
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
                        unsigned::Entry {
                            name: String::from(PID_OWN_FAMILY_NAME),
                            value: ciborium::Value::Text(value.name.into_name_with_prefix()),
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
                            value: ciborium::Value::Text(country.name),
                        }),
                        birth_place.map(|place| unsigned::Entry {
                            name: String::from(PID_BIRTH_CITY),
                            value: ciborium::Value::Text(place.name),
                        }),
                        unsigned::Entry {
                            name: String::from(PID_AGE_OVER_18),
                            value: ciborium::Value::Bool(is_over_18),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_GENDER),
                            value: value.gender.code.into(),
                        }
                        .into(),
                        unsigned::Entry {
                            name: String::from(PID_SPOUSE_OR_PARTNER),
                            value: ciborium::Value::Bool(has_spouse_or_partner),
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
                            value: ciborium::Value::Text(value.residence.address.country.name),
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
                            value: ciborium::Value::Text(value.residence.address.house_number.to_string()),
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

#[derive(Clone, Deserialize)]
pub struct BrpName {
    #[serde(rename = "voornamen")]
    given_names: Option<String>,

    #[serde(rename = "voorvoegsel")]
    family_name_prefix: Option<String>,

    #[serde(rename = "geslachtsnaam")]
    family_name: String,

    #[serde(rename = "aanduidingNaamgebruik")]
    name_usage: Option<BrpNameUsage>,
}

impl BrpName {
    fn info_family_name(self, partner_name: Option<BrpName>) -> Result<String, BrpDataError> {
        let name = match self.name_usage.unwrap_or(BrpNameUsage::Own) {
            BrpNameUsage::Own => self.into_name_with_prefix(),
            BrpNameUsage::Partner => partner_name
                .ok_or(BrpDataError::MissingPartner)?
                .into_name_with_prefix(),
            BrpNameUsage::OwnThenPartner => {
                self.into_combined_name_with_prefix(partner_name.ok_or(BrpDataError::MissingPartner)?)
            }
            BrpNameUsage::PartnerThenOwn => partner_name
                .ok_or(BrpDataError::MissingPartner)?
                .into_combined_name_with_prefix(self),
        };
        Ok(name)
    }

    fn into_name_with_prefix(self) -> String {
        if let Some(prefix) = self.family_name_prefix {
            format!("{} {}", prefix, self.family_name)
        } else {
            self.family_name
        }
    }

    fn into_combined_name_with_prefix(self, other: BrpName) -> String {
        format!("{}-{}", self.into_name_with_prefix(), other.into_name_with_prefix())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "code")]
pub enum BrpNameUsage {
    #[serde(rename = "E")]
    Own,
    #[serde(rename = "P")]
    Partner,
    #[serde(rename = "V")]
    PartnerThenOwn,
    #[serde(rename = "N")]
    OwnThenPartner,
}

#[derive(Deserialize)]
pub struct BrpBirth {
    #[serde(rename = "datum")]
    date: BrpDate,

    #[serde(rename = "land")]
    country: Option<BrpBirthCountry>,

    #[serde(rename = "plaats")]
    place: Option<BrpBirthPlace>,
}

#[derive(Deserialize)]
pub struct BrpDate {
    #[serde(rename = "datum")]
    date: NaiveDate,
}

#[derive(Clone, Deserialize)]
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

#[derive(Clone, Deserialize)]
pub struct BrpPartner {
    #[serde(rename = "soortVerbintenis")]
    kind: BrpMaritalStatus,

    #[serde(rename = "naam")]
    name: BrpName,

    #[serde(rename = "aangaanHuwelijkPartnerschap")]
    start: Option<GbaMarriagePartnershipStart>,

    #[serde(rename = "ontbindingHuwelijkPartnerschap")]
    end: Option<GbaMarriagePartnershipEnd>,
}

#[derive(Clone, Deserialize)]
pub struct GbaMarriagePartnershipStart {}

#[derive(Clone, Deserialize)]
pub struct GbaMarriagePartnershipEnd {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "code")]
pub enum BrpMaritalStatus {
    #[serde(rename = "H")]
    Huwelijk,
    #[serde(rename = "P")]
    GeregistreerdPartnerschap,
    #[serde(rename = ".")]
    Onbekend,
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

    use crate::pid::brp::data::{BrpDataError, BrpPersons};

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

    #[rstest]
    #[case("frouke", "Jansen", "Jansen")]
    #[case("naamgebruik-eigen", "van Buren", "van Buren")]
    #[case("naamgebruik-partner", "Postma", "van Buren")]
    #[case("naamgebruik-partner-first", "ten Hag-van Buren", "van Buren")]
    #[case("naamgebruik-partner-last", "van Buren-ten Hag", "van Buren")]
    #[case("prefix-family-name", "van Buren", "van Buren")]
    fn should_handle_family_name(
        #[case] json_file_name: &str,
        #[case] expected_family_name: &str,
        #[case] expected_own_family_name: &str,
    ) {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json(json_file_name)).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert_eq!(
            expected_family_name,
            brp_person
                .name
                .clone()
                .info_family_name(brp_person.partner().cloned().map(|p| p.name))
                .unwrap()
        );
        assert_eq!(
            expected_own_family_name,
            brp_person.name.clone().into_name_with_prefix()
        );
    }

    #[test]
    fn should_error_for_naamgebruik_partner_without_partner() {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json("naamgebruik-illegal")).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert!(matches!(
            brp_person
                .name
                .clone()
                .info_family_name(brp_person.partner().cloned().map(|p| p.name)),
            Err(BrpDataError::MissingPartner)
        ));
    }

    #[test]
    fn should_be_over_18() {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert!(brp_person.is_over_18());
    }

    #[rstest]
    #[case("married")]
    #[case("remarried")]
    #[case("geregistreerd-partnerschap")]
    fn should_have_spouse_or_partner(#[case] json_file_name: &str) {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json(json_file_name)).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert!(brp_person.has_spouse_or_partner());
    }

    #[rstest]
    #[case("divorced")]
    #[case("frouke")]
    fn should_not_have_spouse_or_partner(#[case] json_file_name: &str) {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json(json_file_name)).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert!(!brp_person.has_spouse_or_partner());
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
    fn should_convert_brp_person_to_mdoc() {
        let mut brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let unsigned_mdoc: Vec<UnsignedMdoc> = brp_persons.persons.remove(0).try_into().unwrap();

        assert_eq!(2, unsigned_mdoc.len());

        let pid_card = &unsigned_mdoc[0];
        let address_card = &unsigned_mdoc[1];

        assert_eq!(
            vec![
                ("bsn", "999991772".into()),
                ("family_name", "Jansen".into()),
                ("own_family_name", "Jansen".into()),
                ("given_name", "Frouke".into()),
                ("birth_date", "2000-03-24".into()),
                ("birth_country", "België".into()),
                ("birth_city", "Luik".into()),
                ("age_over_18", true.into()),
                ("gender", 2.into()),
                ("has_spouse_or_partner", false.into()),
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
