use std::{collections::HashMap, env, path::PathBuf, str::FromStr};

use http::StatusCode;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    gba,
    gba::data::{Categorievoorkomen, GbaResponse},
};

static NATIONALITY_TABLE: Lazy<HashMap<String, String>> =
    Lazy::new(|| read_csv("Tabel32 Nationaliteitentabel (gesorteerd op code)").unwrap());

static MUNICIPALITIES_TABLE: Lazy<HashMap<String, String>> =
    Lazy::new(|| read_csv("Tabel33 Gemeententabel (gesorteerd op code)").unwrap());

static COUNTRIES_TABLE: Lazy<HashMap<String, String>> =
    Lazy::new(|| read_csv("Tabel34 Landentabel (gesorteerd op code)").unwrap());

pub fn initialize_eager() {
    let _ = Lazy::force(&NATIONALITY_TABLE);
    let _ = Lazy::force(&MUNICIPALITIES_TABLE);
    let _ = Lazy::force(&COUNTRIES_TABLE);
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("GBA-V error: {0}")]
    Gba(#[from] gba::error::Error),
}

impl From<&Error> for StatusCode {
    fn from(value: &Error) -> Self {
        match value {
            Error::Gba(e) => e.into(),
        }
    }
}

fn read_csv(name: &str) -> Result<HashMap<String, String>, csv::Error> {
    let mut map = HashMap::new();
    let mut reader = csv::ReaderBuilder::new().has_headers(true).from_path(csv_path(name))?;
    for record in reader.records() {
        let record = record?;
        if let (Some(code), Some(description)) = (record.get(0), record.get(1)) {
            map.insert(String::from(code), String::from(description));
        }
    }
    Ok(map)
}

fn csv_path(name: &str) -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(format!("resources/stamdata/{}.csv", name))
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PersonQuery {
    pub r#type: String,

    #[serde(rename = "burgerservicenummer")]
    pub bsn: Vec<String>,

    #[serde(rename = "gemeente_van_inschrijving")]
    pub registration_municipality: Option<String>,

    pub fields: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PersonsResponse {
    pub r#type: String,
    #[serde(rename = "personen")]
    pub persons: Vec<GbaPerson>,
}

impl PersonsResponse {
    pub fn create(gba_response: GbaResponse) -> Result<Self, Error> {
        Ok(Self {
            r#type: String::from("RaadpleegMetBurgerservicenummer"),
            persons: vec![gba_response.try_into()?],
        })
    }

    pub fn filter_terminated_nationalities(&mut self) {
        self.persons.iter_mut().for_each(|person| {
            let mut keep = vec![];

            while !person.nationalities.is_empty() {
                let nationality = person.nationalities.remove(0);

                // If the current nationality is terminated, it means the previous nationality (to which the current one
                // applies) should be discarded.
                if nationality.reason_termination.is_none() && nationality.nationality.is_some() {
                    keep.push(nationality);
                } else if nationality.reason_termination.is_some() && !keep.is_empty() {
                    keep.pop();
                }
            }

            person.nationalities = keep;
        })
    }
}

/// Converts GBA-V XML to Haal-Centraal JSON. The reference for the category- and element numbers is
/// the Logisch Ontwerp BRP: https://www.rvig.nl/lo-brp
impl TryFrom<GbaResponse> for GbaPerson {
    type Error = Error;

    fn try_from(value: GbaResponse) -> Result<Self, Self::Error> {
        let cat1 = value.get_mandatory_voorkomen(1)?;
        let cat4s: Vec<&Categorievoorkomen> = value.get_voorkomens(4);
        let cat5s: Vec<&Categorievoorkomen> = value.get_voorkomens(5);
        let cat8 = value.get_mandatory_voorkomen(8)?;

        let result = Self {
            bsn: cat1.elementen.get_mandatory("120")?,
            gender: GbaCode {
                code: cat1.elementen.get_mandatory("410")?,
            },
            name: cat1.try_into()?,
            birth: cat1.try_into()?,
            nationalities: cat4s
                .into_iter()
                .map(|cat4| cat4.try_into())
                .collect::<Result<_, Error>>()?,
            partners: cat5s
                .into_iter()
                .map(|cat5| cat5.try_into())
                .collect::<Result<_, Error>>()?,
            address: cat8.try_into()?,
            investigation: cat1
                .elementen
                .get_optional("8310")
                .map(|_| cat1.try_into())
                .transpose()?,
        };

        Ok(result)
    }
}

impl TryFrom<&Categorievoorkomen> for GbaName {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let name = GbaName {
            given_names: value.elementen.get_optional("210"),
            family_name_prefix: value.elementen.get_optional("230"),
            family_name: value.elementen.get_mandatory("240")?,
            name_usage: GbaCode {
                code: value.elementen.get_optional("6110").unwrap_or(String::from("E")),
            },
        };
        Ok(name)
    }
}

impl TryFrom<&Categorievoorkomen> for GbaBirth {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let birth = GbaBirth {
            date: value.elementen.get_mandatory("310")?,
            place: value.elementen.get_optional("320").map(municipalities_code_description),
            country: value.elementen.get_optional("330").map(country_code_description),
        };
        Ok(birth)
    }
}

impl TryFrom<&Categorievoorkomen> for GbaNationality {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let nationality = GbaNationality {
            nationality: value.elementen.get_optional("510").map(nationality_code_description),
            reason_intake: value.elementen.get_optional("6310").map(|code| GbaCode { code }),
            reason_termination: value.elementen.get_optional("6410").map(|code| GbaCode { code }),
            date_start_validity: value.elementen.get_optional("8510"),
            investigation: value
                .elementen
                .get_optional("8310")
                .map(|_| value.try_into())
                .transpose()?,
        };
        Ok(nationality)
    }
}

impl TryFrom<&Categorievoorkomen> for GbaPartner {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let partner = GbaPartner {
            name: value.try_into()?,
            birth: value.try_into()?,
            gender: GbaCode {
                code: value.elementen.get_mandatory("410")?,
            },
            kind: GbaCode {
                code: value.elementen.get_mandatory("1510")?,
            },
            start: value
                .elementen
                .get_optional("610")
                .map(|date| GbaMarriagePartnershipStart {
                    date,
                    place: value.elementen.get_optional("620").map(municipalities_code_description),
                    country: value.elementen.get_optional("630").map(country_code_description),
                }),
            end: value
                .elementen
                .get_optional("710")
                .map(|date| GbaMarriagePartnershipEnd { date }),
            investigation: value
                .elementen
                .get_optional("8310")
                .map(|_| value.try_into())
                .transpose()?,
        };
        Ok(partner)
    }
}

impl TryFrom<&Categorievoorkomen> for GbaAddress {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let address = GbaAddress {
            short_street_name: value.elementen.get_mandatory("1110")?,
            official_street_name: value.elementen.get_mandatory("1115")?,
            house_number: u32::from_str(&value.elementen.get_mandatory("1120")?)
                .expect("housenumber should be an integer"),
            house_letter: value.elementen.get_optional("1130"),
            house_number_addition: value.elementen.get_optional("1140"),
            postal_code: value.elementen.get_optional("1160"),
            city: value.elementen.get_mandatory("1170")?,
            address_function: value.elementen.get_optional("7210").map(|code| GbaCode { code }),
        };
        Ok(address)
    }
}

impl TryFrom<&Categorievoorkomen> for GbaInvestigation {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let investigation = GbaInvestigation {
            data_investigated: value.elementen.get_optional("8310"),
            start_date: value.elementen.get_optional("8320"),
        };
        Ok(investigation)
    }
}

fn municipalities_code_description(code: String) -> GbaCodeDescription {
    let (code, description) = if Regex::new(r"^\d{4}$").unwrap().is_match(&code) {
        let description = MUNICIPALITIES_TABLE.get(&code).cloned();
        (Some(code), description)
    } else {
        (None, Some(code))
    };
    GbaCodeDescription { code, description }
}

fn country_code_description(code: String) -> GbaCodeDescription {
    let description = COUNTRIES_TABLE.get(&code).cloned();
    GbaCodeDescription {
        code: Some(code),
        description,
    }
}

fn nationality_code_description(code: String) -> GbaCodeDescription {
    let description = NATIONALITY_TABLE.get(&code).cloned();
    GbaCodeDescription {
        code: Some(code),
        description,
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GbaPerson {
    #[serde(rename = "burgerservicenummer")]
    bsn: String,

    #[serde(rename = "geslacht")]
    gender: GbaCode,

    #[serde(rename = "naam")]
    name: GbaName,

    #[serde(rename = "geboorte")]
    birth: GbaBirth,

    #[serde(rename = "persoonInOnderzoek")]
    investigation: Option<GbaInvestigation>,

    #[serde(rename = "partners")]
    partners: Vec<GbaPartner>,

    #[serde(rename = "nationaliteiten")]
    nationalities: Vec<GbaNationality>,

    #[serde(rename = "verblijfplaats")]
    address: GbaAddress,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GbaName {
    #[serde(rename = "voornamen")]
    given_names: Option<String>,

    #[serde(rename = "voorvoegsel")]
    family_name_prefix: Option<String>,

    #[serde(rename = "geslachtsnaam")]
    family_name: String,

    #[serde(rename = "aanduidingNaamgebruik")]
    name_usage: GbaCode,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GbaBirth {
    #[serde(rename = "datum")]
    date: String,

    #[serde(rename = "land")]
    country: Option<GbaCodeDescription>,

    #[serde(rename = "plaats")]
    place: Option<GbaCodeDescription>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GbaResidence {
    #[serde(rename = "verblijfadres")]
    address: GbaAddress,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GbaAddress {
    #[serde(rename = "straat")]
    short_street_name: String,

    #[serde(rename = "naamOpenbareRuimte")]
    official_street_name: String,

    #[serde(rename = "huisnummer")]
    house_number: u32,

    #[serde(rename = "huisletter")]
    house_letter: Option<String>,

    #[serde(rename = "huisnummertoevoeging")]
    house_number_addition: Option<String>,

    #[serde(rename = "postcode")]
    postal_code: Option<String>,

    #[serde(rename = "woonplaats")]
    city: String,

    #[serde(rename = "functieAdres")]
    address_function: Option<GbaCode>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GbaNationality {
    #[serde(rename = "nationaliteit")]
    nationality: Option<GbaCodeDescription>,

    #[serde(skip)]
    reason_termination: Option<GbaCode>,

    #[serde(rename = "redenOpname")]
    reason_intake: Option<GbaCode>,

    #[serde(rename = "inOnderzoek")]
    investigation: Option<GbaInvestigation>,

    #[serde(rename = "datumIngangGeldigheid")]
    date_start_validity: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GbaPartner {
    #[serde(rename = "geslacht")]
    gender: GbaCode,

    #[serde(rename = "naam")]
    name: GbaName,

    #[serde(rename = "geboorte")]
    birth: GbaBirth,

    #[serde(rename = "soortVerbintenis")]
    kind: GbaCode,

    #[serde(rename = "aangaanHuwelijkPartnerschap")]
    start: Option<GbaMarriagePartnershipStart>,

    #[serde(rename = "ontbindingHuwelijkPartnerschap")]
    end: Option<GbaMarriagePartnershipEnd>,

    #[serde(rename = "inOnderzoek")]
    investigation: Option<GbaInvestigation>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GbaMarriagePartnershipStart {
    #[serde(rename = "datum")]
    date: String,

    #[serde(rename = "plaats")]
    place: Option<GbaCodeDescription>,

    #[serde(rename = "land")]
    country: Option<GbaCodeDescription>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GbaMarriagePartnershipEnd {
    #[serde(rename = "datum")]
    date: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GbaCodeDescription {
    code: Option<String>,
    #[serde(rename = "omschrijving")]
    description: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GbaCode {
    code: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GbaInvestigation {
    #[serde(rename = "aanduidingGegevensInOnderzoek")]
    data_investigated: Option<String>,

    #[serde(rename = "datumIngangOnderzoek")]
    start_date: Option<String>,
}
