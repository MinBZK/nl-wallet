use std::{collections::HashMap, env, path::PathBuf, str::FromStr, sync::LazyLock};

use nutype::nutype;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    gba,
    gba::data::{Categorievoorkomen, GbaResponse},
};

static NATIONALITY_TABLE: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| read_csv("Tabel32 Nationaliteitentabel (gesorteerd op code)").unwrap());

static MUNICIPALITIES_TABLE: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| read_csv("Tabel33 Gemeententabel (gesorteerd op code)").unwrap());

static COUNTRIES_TABLE: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| read_csv("Tabel34 Landentabel (gesorteerd op code)").unwrap());

pub enum Category {
    Person = 1,
    Nationality = 4,
    MarriagePartnership = 5,
    Address = 8,
}

impl Category {
    pub fn code(self) -> u8 {
        self as u8
    }
}

pub enum Element {
    Bsn = 120,
    Voornamen = 210,
    Voorvoegsel = 230,
    Geslachtsnaam = 240,
    Geboortedatum = 310,
    Geboorteplaats = 320,
    Geboorteland = 330,
    Geslacht = 410,
    Nationality = 510,
    DatumAangaanHuwelijk = 610,
    PlaatsAangaanHuwelijk = 620,
    LandAangaanHuwelijk = 630,
    DatumOntbindingHuwelijk = 710,
    Straatnaam = 1110,
    NaamOpenbareRuimte = 1115,
    Huisnummer = 1120,
    Huisletter = 1130,
    Huisnummertoevoeging = 1140,
    Postcode = 1160,
    Woonplaats = 1170,
    SoortVerbintenis = 1510,
    IndicatieNaamgebruik = 6110,
    RedenOpnameNationaliteit = 6310,
    RedenBeeindigenNationaliteit = 6410,
    AanduidingBijzonderNederlanderschap = 6510,
    AangifteAdreshouding = 7210,
    AanduidingInOnderzoek = 8310,
    DatumIngangOnderzoek = 8320,
    IngangsdatumGeldigheid = 8510,
}

impl Element {
    pub fn code(self) -> u16 {
        self as u16
    }
}

pub fn initialize_eager() {
    let _ = LazyLock::force(&NATIONALITY_TABLE);
    let _ = LazyLock::force(&MUNICIPALITIES_TABLE);
    let _ = LazyLock::force(&COUNTRIES_TABLE);
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("GBA-V error: {0}")]
    Gba(#[from] gba::error::Error),
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

#[nutype(
    validate(predicate = Bsn::validate),
    derive(Deserialize, Serialize, Clone, Debug, Display, AsRef)
)]
pub struct Bsn(String);

impl Bsn {
    // Validate the BSN by using the so-called "Elfproef".
    // See section I.2.3 of Logisch Ontwerp BSN 2024 Q1 for the details on the "Elfproef"
    fn validate(bsn: &str) -> bool {
        // Pad the BSN with a leading zero when the length is 8
        let padded_bsn = format!("{:0>9}", bsn);

        let digits: Vec<i32> = padded_bsn
            .chars()
            .filter_map(|c| c.to_digit(10))
            .map(|d| d as i32)
            .collect();

        if digits.len() != 9 {
            return false;
        }

        let weights = [9, 8, 7, 6, 5, 4, 3, 2, -1];

        let sum: i32 = digits.iter().zip(weights).map(|(digit, weight)| digit * weight).sum();

        sum % 11 == 0
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PersonQuery {
    pub r#type: String,

    #[serde(rename = "burgerservicenummer")]
    pub bsn: Vec<Bsn>,

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
            persons: gba_response.try_into()?,
        })
    }

    // We get a list of nationalities, of which an entry can just be that a nationality (doesn't say which one) is
    // terminated. The assumption here (taken from the Haal-Centraal code) is that this termination applies to the
    // previous nationality in the list.
    pub fn filter_terminated_nationalities(&mut self) {
        self.persons.iter_mut().for_each(|person| {
            let mut keep = vec![];

            while !person.nationalities.is_empty() {
                let nationality = person.nationalities.remove(0);

                // If the current nationality is terminated, the previous nationality should be discarded.
                if nationality.reason_termination.is_none() && nationality.nationality.is_some() {
                    keep.push(nationality);
                } else if nationality.reason_termination.is_some() && !keep.is_empty() {
                    keep.pop();
                }
            }

            person.nationalities = keep;
        });
    }
}

/// Converts GBA-V XML to Haal-Centraal JSON. The reference for the category- and element numbers is
/// the Logisch Ontwerp BRP: https://www.rvig.nl/lo-brp
impl TryFrom<GbaResponse> for Vec<GbaPerson> {
    type Error = Error;

    fn try_from(value: GbaResponse) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(vec![]);
        }

        let cat1 = value.get_mandatory_voorkomen(Category::Person.code())?;
        let cat4s: Vec<&Categorievoorkomen> = value.get_voorkomens(Category::Nationality.code());
        let cat5s: Vec<&Categorievoorkomen> = value.get_voorkomens(Category::MarriagePartnership.code());
        let cat8 = value.get_mandatory_voorkomen(Category::Address.code())?;

        let person = GbaPerson {
            bsn: cat1.elementen.get_mandatory(Element::Bsn.code())?,
            gender: GbaCode {
                code: cat1.elementen.get_mandatory(Element::Geslacht.code())?,
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
                .get_optional(Element::AanduidingInOnderzoek.code())
                .map(|_| cat1.try_into())
                .transpose()?,
        };

        Ok(vec![person])
    }
}

impl TryFrom<&Categorievoorkomen> for GbaName {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let name = GbaName {
            given_names: value.elementen.get_optional(Element::Voornamen.code()),
            family_name_prefix: value.elementen.get_optional(Element::Voorvoegsel.code()),
            family_name: value.elementen.get_mandatory(Element::Geslachtsnaam.code())?,
            name_usage: GbaCode {
                code: value
                    .elementen
                    .get_optional(Element::IndicatieNaamgebruik.code())
                    .unwrap_or(String::from("E")),
            },
        };
        Ok(name)
    }
}

impl TryFrom<&Categorievoorkomen> for GbaBirth {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let birth = GbaBirth {
            date: value.elementen.get_mandatory(Element::Geboortedatum.code())?,
            place: value
                .elementen
                .get_optional(Element::Geboorteplaats.code())
                .map(municipalities_code_description),
            country: value
                .elementen
                .get_optional(Element::Geboorteland.code())
                .map(country_code_description),
        };
        Ok(birth)
    }
}

impl TryFrom<&Categorievoorkomen> for GbaNationality {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let nationality = GbaNationality {
            nationality: value
                .elementen
                .get_optional(Element::Nationality.code())
                .map(nationality_code_description),
            indication_special_citizenship: value
                .elementen
                .get_optional(Element::AanduidingBijzonderNederlanderschap.code()),
            reason_intake: value
                .elementen
                .get_optional(Element::RedenOpnameNationaliteit.code())
                .map(|code| GbaCode { code }),
            reason_termination: value
                .elementen
                .get_optional(Element::RedenBeeindigenNationaliteit.code())
                .map(|code| GbaCode { code }),
            date_start_validity: value.elementen.get_optional(Element::IngangsdatumGeldigheid.code()),
            investigation: value
                .elementen
                .get_optional(Element::AanduidingInOnderzoek.code())
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
            birth: value
                .elementen
                .get_optional(Element::Geboortedatum.code())
                .map(|_| value.try_into())
                .transpose()?,
            gender: value
                .elementen
                .get_optional(Element::Geslacht.code())
                .map(|geslacht| GbaCode { code: geslacht }),
            kind: GbaCode {
                code: value.elementen.get_mandatory(Element::SoortVerbintenis.code())?,
            },
            start: value
                .elementen
                .get_optional(Element::DatumAangaanHuwelijk.code())
                .map(|date| GbaMarriagePartnershipStart {
                    date,
                    place: value
                        .elementen
                        .get_optional(Element::PlaatsAangaanHuwelijk.code())
                        .map(municipalities_code_description),
                    country: value
                        .elementen
                        .get_optional(Element::LandAangaanHuwelijk.code())
                        .map(country_code_description),
                }),
            end: value
                .elementen
                .get_optional(Element::DatumOntbindingHuwelijk.code())
                .map(|date| GbaMarriagePartnershipEnd { date }),
            investigation: value
                .elementen
                .get_optional(Element::AanduidingInOnderzoek.code())
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
            short_street_name: value.elementen.get_mandatory(Element::Straatnaam.code())?,
            official_street_name: value.elementen.get_mandatory(Element::NaamOpenbareRuimte.code())?,
            house_number: u32::from_str(&value.elementen.get_mandatory(Element::Huisnummer.code())?)
                .expect("housenumber should be an integer"),
            house_letter: value.elementen.get_optional(Element::Huisletter.code()),
            house_number_addition: value.elementen.get_optional(Element::Huisnummertoevoeging.code()),
            postal_code: value.elementen.get_optional(Element::Postcode.code()),
            city: value.elementen.get_mandatory(Element::Woonplaats.code())?,
            address_function: value
                .elementen
                .get_optional(Element::AangifteAdreshouding.code())
                .map(|code| GbaCode { code }),
        };
        Ok(address)
    }
}

impl TryFrom<&Categorievoorkomen> for GbaInvestigation {
    type Error = Error;

    fn try_from(value: &Categorievoorkomen) -> Result<Self, Self::Error> {
        let investigation = GbaInvestigation {
            data_investigated: value.elementen.get_optional(Element::AanduidingInOnderzoek.code()),
            start_date: value.elementen.get_optional(Element::DatumIngangOnderzoek.code()),
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

    #[serde(rename = "aanduidingBijzonderNederlanderschap")]
    indication_special_citizenship: Option<String>,

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
    gender: Option<GbaCode>,

    #[serde(rename = "naam")]
    name: GbaName,

    #[serde(rename = "geboorte")]
    birth: Option<GbaBirth>,

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

#[cfg(test)]
mod test {
    use crate::haal_centraal::Bsn;
    use rstest::rstest;

    #[rstest]
    #[case("999991772")]
    #[case("900253010")]
    #[case("900265462")]
    #[case("11122146")]
    fn test_bsn_should_validate(#[case] bsn: &str) {
        assert!(Bsn::validate(bsn));
    }

    #[rstest]
    #[case("999991773")]
    #[case("900253011")]
    #[case("900265463")]
    #[case("900265")]
    #[case("abcd")]
    #[case("abcdefghi")]
    #[case("11122234")]
    fn test_bsn_should_not_validate(#[case] bsn: &str) {
        assert!(!Bsn::validate(bsn));
    }
}
