use attestation_data::attributes::Attribute;
use attestation_data::attributes::Attributes;
use chrono::NaiveDate;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde_with::skip_serializing_none;

use crate::pid::rulebook::constants::PID_BIRTH_PLACE_GROUP;
use crate::pid::rulebook::constants::PID_BSN;
use crate::pid::rulebook::constants::PID_FAMILY_NAME;
use crate::pid::rulebook::constants::PID_GIVEN_NAME;
use crate::pid::rulebook::constants::PID_MDOC_BIRTH_DATE;
use crate::pid::rulebook::constants::PID_MDOC_FAMILY_NAME_BIRTH;
use crate::pid::rulebook::constants::PID_MDOC_NAMESPACE;
use crate::pid::rulebook::constants::PID_MDOC_NATIONALITY;
use crate::pid::rulebook::constants::PID_MDOC_RESIDENT_CITY;
use crate::pid::rulebook::constants::PID_MDOC_RESIDENT_COUNTRY;
use crate::pid::rulebook::constants::PID_MDOC_RESIDENT_POSTAL_CODE;
use crate::pid::rulebook::constants::PID_MDOC_RESIDENT_STREET;
use crate::pid::rulebook::constants::PID_RECOVERY_CODE;
use crate::pid::rulebook::constants::PID_SD_JWT_ADDRESS_GROUP;
use crate::pid::rulebook::constants::PID_SD_JWT_BIRTH_DATE;
use crate::pid::rulebook::constants::PID_SD_JWT_COUNTRY;
use crate::pid::rulebook::constants::PID_SD_JWT_FAMILY_NAME_BIRTH;
use crate::pid::rulebook::constants::PID_SD_JWT_HOUSE_NUMBER;
use crate::pid::rulebook::constants::PID_SD_JWT_LOCALITY;
use crate::pid::rulebook::constants::PID_SD_JWT_NATIONALITIES;
use crate::pid::rulebook::constants::PID_SD_JWT_POSTAL_CODE;
use crate::pid::rulebook::constants::PID_SD_JWT_STREET_ADDRESS;
use crate::pid::rulebook::constants::PID_SEX;
use crate::pid::rulebook::country_code::IsoCountryCode;
use crate::pid::rulebook::country_code::RulebookCountryCode;
use crate::pid::rulebook::country_code::RulebookNationality;
use crate::pid::rulebook::issuer_metadata::PidIssuerMetadata;
use crate::pid::rulebook::sex::IsoSex;
use crate::pid::rulebook::sex::RulebookSex;

#[derive(Deserialize)]
pub struct BrpPersons {
    #[serde(rename = "personen")]
    pub persons: Vec<BrpPerson>,
}

// Represents a person from the BRP.
// Note: for categories that can occur multiple times, the ordering is such that the most recent category is first.
// See Logisch Ontwerp BRP 2024 Q2 section 5.1.7.3
#[derive(Clone, Deserialize)]
pub struct BrpPerson {
    #[serde(rename = "burgerservicenummer")]
    bsn: String,

    #[serde(rename = "geslacht")]
    gender: Option<BrpGender>,

    #[serde(rename = "naam")]
    name: BrpName,

    #[serde(rename = "geboorte")]
    birth: BrpBirth,

    #[serde(rename = "nationaliteiten", default)]
    nationalities: Vec<BrpNationality>,

    #[serde(rename = "verblijfplaats")]
    residence: BrpResidence,

    #[serde(rename = "partners", default)]
    partners: Vec<BrpPartner>,
}

impl BrpPerson {
    pub fn bsn(&self) -> &str {
        &self.bsn
    }

    fn current_partner(&self) -> Option<&BrpPartner> {
        self.partners
            .iter()
            .find(|partner| partner.start.is_some() && partner.end.is_none())
    }

    // Computes the family name as it should be displayed (including partner name, if applicable), falls back to the own
    // name
    fn family_name(&self) -> String {
        let own_name = || self.name.name_with_prefix();
        let partner_name = || self.current_partner().map(|partner| partner.name.name_with_prefix());

        match self.name.name_usage.code {
            BrpNameUsageCode::Own => own_name(),
            BrpNameUsageCode::Partner => partner_name().unwrap_or_else(own_name),
            BrpNameUsageCode::OwnThenPartner => partner_name()
                .map(|partner| format!("{}-{partner}", own_name()))
                .unwrap_or_else(own_name),
            BrpNameUsageCode::PartnerThenOwn => partner_name()
                .map(|partner| format!("{partner}-{}", own_name()))
                .unwrap_or_else(own_name),
        }
    }
}

impl BrpPerson {
    pub fn into_sd_jwt_attributes(self, recovery_code: Attribute, issuer_metadata: PidIssuerMetadata) -> Attributes {
        let address_attributes = self.residence.clone().into_sd_jwt_address_attributes();
        let (mut attributes, birth_date, nationality, family_name_birth) = self.common_attributes();

        attributes.insert(
            String::from(PID_SD_JWT_BIRTH_DATE),
            Attribute::Text(birth_date.format("%Y-%m-%d").to_string()), // ISO-8601 string
        );
        attributes.insert(String::from(PID_SD_JWT_NATIONALITIES), nationality);
        attributes.insert(String::from(PID_SD_JWT_FAMILY_NAME_BIRTH), family_name_birth);
        attributes.extend(issuer_metadata.into_attributes());
        attributes.insert(String::from(PID_RECOVERY_CODE), recovery_code);
        attributes.insert(
            String::from(PID_SD_JWT_ADDRESS_GROUP),
            Attribute::Object(IndexMap::from_iter(address_attributes)),
        );

        Attributes::from(attributes)
    }

    pub fn into_mdoc_attributes(self, recovery_code: Attribute, issuer_metadata: PidIssuerMetadata) -> Attributes {
        let address_attributes = self.residence.clone().into_mdoc_address_attributes();
        let (mut attributes, birth_date, nationality, family_name_birth) = self.common_attributes();

        // In mdoc, `birth_date` should be encoded as `full-date` (CBOR tag 1004)
        attributes.insert(PID_MDOC_BIRTH_DATE.to_owned(), Attribute::Date(birth_date));
        attributes.insert(PID_MDOC_NATIONALITY.to_owned(), nationality);
        attributes.insert(PID_MDOC_FAMILY_NAME_BIRTH.to_owned(), family_name_birth);
        attributes.extend(issuer_metadata.into_attributes());
        attributes.insert(String::from(PID_RECOVERY_CODE), recovery_code);
        attributes.extend(address_attributes);

        Attributes::from(IndexMap::from([(
            PID_MDOC_NAMESPACE.to_owned(),
            Attribute::Object(attributes),
        )]))
    }

    /// Everything but `birth_date`, `nationality` and `family_name_birth`, since those have different key/value
    /// formats or keys for mdoc and SD-JWT.
    fn common_attributes(self) -> (IndexMap<String, Attribute>, NaiveDate, Attribute, Attribute) {
        let given_names = self.name.given_names.clone();
        let family_name = self.family_name();
        let family_name_birth = Attribute::Text(self.name.name_with_prefix());
        let sex = self
            .gender
            .map(|gender| Attribute::from(RulebookSex::from(gender.code)));
        let birth_date = self.birth.date.date;
        let place_of_birth = self.birth.into_place_of_birth_attribute();
        let nationality = Attribute::Array(
            self.nationalities
                .into_iter()
                .filter_map(|nationality| nationality.nationality)
                .map(|detail| Attribute::from(RulebookNationality::from(detail.code)))
                .collect_vec(),
        );

        let attributes = IndexMap::from_iter(
            vec![
                Some((String::from(PID_FAMILY_NAME), Attribute::Text(family_name))),
                given_names.map(|names| (String::from(PID_GIVEN_NAME), Attribute::Text(names))),
                Some((String::from(PID_BSN), Attribute::Text(self.bsn))),
                sex.map(|sex| (PID_SEX.to_owned(), sex)),
                place_of_birth.map(|attribute| (PID_BIRTH_PLACE_GROUP.to_owned(), attribute)),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<(String, Attribute)>>(),
        );

        (attributes, birth_date, nationality, family_name_birth)
    }
}

impl BrpResidence {
    pub fn into_sd_jwt_address_attributes(self) -> impl Iterator<Item = (String, Attribute)> {
        vec![
            self.address.street().map(|street| {
                (
                    String::from(PID_SD_JWT_STREET_ADDRESS),
                    Attribute::Text(String::from(street)),
                )
            }),
            Some((
                String::from(PID_SD_JWT_HOUSE_NUMBER),
                Attribute::Text(self.address.locator_designator()),
            )),
            Some((
                String::from(PID_SD_JWT_POSTAL_CODE),
                Attribute::Text(self.address.postal_code),
            )),
            Some((String::from(PID_SD_JWT_LOCALITY), Attribute::Text(self.address.city))),
            self.address
                .country
                .code
                .to_rulebook_country()
                .map(|country| (PID_SD_JWT_COUNTRY.to_owned(), Attribute::from(country))),
        ]
        .into_iter()
        .flatten()
    }

    /// mdoc has no nested `address` and uses dedicated attribute identifiers (`resident_*`)
    pub fn into_mdoc_address_attributes(self) -> impl Iterator<Item = (String, Attribute)> {
        vec![
            self.address
                .mdoc_street()
                .map(|street| (PID_MDOC_RESIDENT_STREET.to_owned(), Attribute::Text(street))),
            Some((
                PID_MDOC_RESIDENT_POSTAL_CODE.to_owned(),
                Attribute::Text(self.address.postal_code),
            )),
            Some((PID_MDOC_RESIDENT_CITY.to_owned(), Attribute::Text(self.address.city))),
            self.address
                .country
                .code
                .to_rulebook_country()
                .map(|country| (PID_MDOC_RESIDENT_COUNTRY.to_owned(), Attribute::from(country))),
        ]
        .into_iter()
        .flatten()
    }
}

#[derive(Clone, Deserialize)]
pub struct BrpGender {
    code: BrpGenderCode,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all(deserialize = "UPPERCASE"))]
pub enum BrpGenderCode {
    V,
    M,
    O,
}

/// BRP's `geslacht.code` only ever yields "onbekend", "man" or "vrouw", all three of which fall within the
/// ISO/IEC 5218 base standard, even though the Rulebook's `sex` attribute defines a larger value set (see
/// [`RulebookSex`]).
impl From<BrpGenderCode> for RulebookSex {
    fn from(value: BrpGenderCode) -> Self {
        let iso = match value {
            BrpGenderCode::O => IsoSex::NotKnown,
            BrpGenderCode::M => IsoSex::Male,
            BrpGenderCode::V => IsoSex::Female,
        };

        RulebookSex::Iso(iso)
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
    name_usage: BrpNameUsage,
}

impl BrpName {
    fn name_with_prefix(&self) -> String {
        match &self.family_name_prefix {
            Some(prefix) => format!("{prefix} {}", self.family_name),
            None => self.family_name.clone(),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct BrpNameUsage {
    code: BrpNameUsageCode,
}

#[derive(Clone, Copy, Deserialize)]
pub enum BrpNameUsageCode {
    #[serde(rename = "P")]
    Partner,
    #[serde(rename = "N")]
    OwnThenPartner,
    #[serde(rename = "V")]
    PartnerThenOwn,
    #[serde(rename = "E")]
    Own,
}

#[derive(Clone, Deserialize)]
pub struct BrpPartner {
    #[serde(rename = "naam")]
    name: BrpName,

    #[serde(rename = "aangaanHuwelijkPartnerschap")]
    start: Option<BrpPartnershipEvent>,

    #[serde(rename = "ontbindingHuwelijkPartnerschap")]
    end: Option<BrpPartnershipEvent>,
}

// Only presence/absence is relevant for determining the current partner; its contents are unused.
#[derive(Clone, Deserialize)]
pub struct BrpPartnershipEvent {}

#[derive(Clone, Deserialize)]
pub struct BrpBirth {
    #[serde(rename = "datum")]
    date: BrpDate,

    #[serde(rename = "land")]
    country: Option<BrpCountryDetail>,

    #[serde(rename = "plaats")]
    place: Option<BrpDescription>,
}

impl BrpBirth {
    /// Builds the `place_of_birth` attribute
    ///
    /// As region is not available, a nested map with only `country` and `locality`. Returns `None` if neither is
    /// available.
    ///
    /// ```cddl
    /// cddl place_of_birth = {
    ///     ? "country": tstr  ; a single alpha-2 country code as specified in [ISO 3166-1]
    ///     ? "region": tstr   ; the name of a state, province, district, or local area
    ///     ? "locality": tstr ; the name of a municipality, city, town, or village
    /// }
    /// ```
    fn into_place_of_birth_attribute(self) -> Option<Attribute> {
        let country = self.country.and_then(|detail| detail.code.to_rulebook_country());
        let locality = self.place.map(|description| description.description);

        let group = [
            country.map(|country| (PID_SD_JWT_COUNTRY.to_owned(), Attribute::from(country))),
            locality.map(|locality| (PID_SD_JWT_LOCALITY.to_owned(), Attribute::Text(locality))),
        ]
        .into_iter()
        .flatten()
        .collect::<IndexMap<_, _>>();

        // place_of_birth SHALL contain at least one of the following key-value pairs: "country", "region", or
        // "locality".
        (!group.is_empty()).then_some(Attribute::Object(group))
    }
}

#[derive(Clone, Deserialize)]
pub struct BrpDate {
    #[serde(rename = "datum")]
    date: NaiveDate,
}

#[derive(Clone, Deserialize)]
pub struct BrpDescription {
    #[serde(rename = "omschrijving")]
    description: String,
}

#[derive(Clone, Deserialize)]
pub struct BrpCountryDetail {
    #[serde(rename = "code")]
    code: RvigCountryCode,
}

#[skip_serializing_none]
#[derive(Clone, Deserialize)]
pub struct BrpNationality {
    #[serde(rename = "nationaliteit")]
    nationality: Option<BrpNationalityDetail>,
}

#[derive(Clone, Deserialize)]
pub struct BrpNationalityDetail {
    #[serde(rename = "code")]
    code: RvigNationalityCode,
}

/// RvIG's Tabel 32 Nationaliteitentabel codes. Source: <https://publicaties.rvig.nl/media/13281/download>. `Overig` is
/// a `serde(other)` catch-all for any future code not yet in this enum.
#[derive(Clone, Deserialize)]
pub enum RvigNationalityCode {
    #[serde(rename = "0000")]
    Onbekend,
    #[serde(rename = "0001")]
    Nederlandse,
    #[serde(rename = "0002")]
    BehandeldAlsNederlander,
    #[serde(rename = "0027")]
    Slowaakse,
    #[serde(rename = "0028")]
    Tsjechische,
    #[serde(rename = "0029")]
    BurgerVanBosnieHerzegovina,
    #[serde(rename = "0030")]
    Georgische,
    #[serde(rename = "0031")]
    Turkmeense,
    #[serde(rename = "0032")]
    Tadzjiekse,
    #[serde(rename = "0033")]
    Oezbeekse,
    #[serde(rename = "0034")]
    Oekrainense,
    #[serde(rename = "0035")]
    Kirgizische,
    #[serde(rename = "0036")]
    Moldavische,
    #[serde(rename = "0037")]
    Kazachse,
    #[serde(rename = "0038")]
    Belarussische,
    #[serde(rename = "0039")]
    Azerbeidzjaanse,
    #[serde(rename = "0040")]
    Armeense,
    #[serde(rename = "0041")]
    Russische,
    #[serde(rename = "0042")]
    Sloveense,
    #[serde(rename = "0043")]
    Kroatische,
    #[serde(rename = "0044")]
    Letse,
    #[serde(rename = "0045")]
    Estische,
    #[serde(rename = "0046")]
    Litouwse,
    #[serde(rename = "0047")]
    Marshalleilandse,
    #[serde(rename = "0048")]
    Myanmarese,
    #[serde(rename = "0049")]
    Namibische,
    #[serde(rename = "0050")]
    Albanese,
    #[serde(rename = "0051")]
    Andorrese,
    #[serde(rename = "0052")]
    Belgische,
    #[serde(rename = "0053")]
    Bulgaarse,
    #[serde(rename = "0054")]
    Deense,
    #[serde(rename = "0055")]
    BurgerVanDeBondsrepubliekDuitsland,
    #[serde(rename = "0056")]
    Finse,
    #[serde(rename = "0057")]
    Franse,
    #[serde(rename = "0058")]
    Jemenitische,
    #[serde(rename = "0059")]
    Griekse,
    #[serde(rename = "0060")]
    BritsBurger,
    #[serde(rename = "0061")]
    Hongaarse,
    #[serde(rename = "0062")]
    Ierse,
    #[serde(rename = "0063")]
    IJslandse,
    #[serde(rename = "0064")]
    Italiaanse,
    #[serde(rename = "0065")]
    Joegoslavische,
    #[serde(rename = "0066")]
    Liechtensteinse,
    #[serde(rename = "0067")]
    Luxemburgse,
    #[serde(rename = "0068")]
    Maltese,
    #[serde(rename = "0069")]
    Monegaskische,
    #[serde(rename = "0070")]
    Noorse,
    #[serde(rename = "0071")]
    Oostenrijkse,
    #[serde(rename = "0072")]
    Poolse,
    #[serde(rename = "0073")]
    Portugese,
    #[serde(rename = "0074")]
    Roemeense,
    #[serde(rename = "0075")]
    BurgerVanDeSovjetUnie,
    #[serde(rename = "0076")]
    SanMarinese,
    #[serde(rename = "0077")]
    Spaanse,
    #[serde(rename = "0078")]
    TsjechoSlowaakse,
    #[serde(rename = "0079")]
    Vaticaanse,
    #[serde(rename = "0080")]
    Zweedse,
    #[serde(rename = "0081")]
    Zwitserse,
    #[serde(rename = "0082")]
    OostDuitse,
    #[serde(rename = "0083")]
    BritsOnderdaan,
    #[serde(rename = "0084")]
    Eritrese,
    #[serde(rename = "0085")]
    BritsOverzeesBurger,
    #[serde(rename = "0086")]
    Macedonische,
    #[serde(rename = "0087")]
    Kosovaarse,
    #[serde(rename = "0088")]
    BurgerVanNoordMacedonie,
    #[serde(rename = "0100")]
    Algerijnse,
    #[serde(rename = "0101")]
    Angolese,
    #[serde(rename = "0104")]
    Burundese,
    #[serde(rename = "0105")]
    Botswaanse,
    #[serde(rename = "0106")]
    Burkinese,
    #[serde(rename = "0108")]
    CentraalAfrikaanse,
    #[serde(rename = "0109")]
    Comorese,
    #[serde(rename = "0110")]
    BurgerVanCongo,
    #[serde(rename = "0111")]
    Beninse,
    #[serde(rename = "0112")]
    Egyptische,
    #[serde(rename = "0113")]
    EquatoriaalGuinese,
    #[serde(rename = "0114")]
    Ethiopische,
    #[serde(rename = "0115")]
    Djiboutiaanse,
    #[serde(rename = "0116")]
    Gabonese,
    #[serde(rename = "0117")]
    Gambiaanse,
    #[serde(rename = "0118")]
    Ghanese,
    #[serde(rename = "0119")]
    Guinese,
    #[serde(rename = "0120")]
    Ivoriaanse,
    #[serde(rename = "0121")]
    Kaapverdische,
    #[serde(rename = "0122")]
    Kameroense,
    #[serde(rename = "0123")]
    Kenyaanse,
    #[serde(rename = "0124")]
    Zairese,
    #[serde(rename = "0125")]
    Lesothaanse,
    #[serde(rename = "0126")]
    Liberiaanse,
    #[serde(rename = "0127")]
    Libische,
    #[serde(rename = "0128")]
    Malagassische,
    #[serde(rename = "0129")]
    Malawische,
    #[serde(rename = "0130")]
    Malinese,
    #[serde(rename = "0131")]
    Marokkaanse,
    #[serde(rename = "0132")]
    Mauritaanse,
    #[serde(rename = "0133")]
    Mauritiaanse,
    #[serde(rename = "0134")]
    Mozambikaanse,
    #[serde(rename = "0135")]
    Swazische,
    #[serde(rename = "0136")]
    Nigerese,
    #[serde(rename = "0137")]
    Nigeriaanse,
    #[serde(rename = "0138")]
    Ugandese,
    #[serde(rename = "0139")]
    GuineeBissause,
    #[serde(rename = "0140")]
    ZuidAfrikaanse,
    #[serde(rename = "0141")]
    Eswatinische,
    #[serde(rename = "0142")]
    Zimbabwaanse,
    #[serde(rename = "0143")]
    Rwandese,
    #[serde(rename = "0144")]
    BurgerVanSaoTomeEnPrincipe,
    #[serde(rename = "0145")]
    Senegalese,
    #[serde(rename = "0147")]
    SierraLeoonse,
    #[serde(rename = "0148")]
    Soedanese,
    #[serde(rename = "0149")]
    Somalische,
    #[serde(rename = "0151")]
    Tanzaniaanse,
    #[serde(rename = "0152")]
    Togolese,
    #[serde(rename = "0154")]
    Tsjadische,
    #[serde(rename = "0155")]
    Tunesische,
    #[serde(rename = "0156")]
    Zambiaanse,
    #[serde(rename = "0157")]
    ZuidSoedanese,
    #[serde(rename = "0200")]
    Bahamaanse,
    #[serde(rename = "0202")]
    Belizaanse,
    #[serde(rename = "0204")]
    Canadese,
    #[serde(rename = "0205")]
    CostaRicaanse,
    #[serde(rename = "0206")]
    Cubaanse,
    #[serde(rename = "0207")]
    Dominicaanse,
    #[serde(rename = "0208")]
    Salvadoraanse,
    #[serde(rename = "0211")]
    Guatemalaanse,
    #[serde(rename = "0212")]
    Haitiaanse,
    #[serde(rename = "0213")]
    Hondurese,
    #[serde(rename = "0214")]
    Jamaicaanse,
    #[serde(rename = "0216")]
    Mexicaanse,
    #[serde(rename = "0218")]
    Nicaraguaanse,
    #[serde(rename = "0219")]
    Panamese,
    #[serde(rename = "0222")]
    BurgerVanTrinidadEnTobago,
    #[serde(rename = "0223")]
    AmerikaansBurger,
    #[serde(rename = "0250")]
    Argentijnse,
    #[serde(rename = "0251")]
    Barbadaanse,
    #[serde(rename = "0252")]
    Boliviaanse,
    #[serde(rename = "0253")]
    Braziliaanse,
    #[serde(rename = "0254")]
    Chileense,
    #[serde(rename = "0255")]
    Colombiaanse,
    #[serde(rename = "0256")]
    Ecuadoraanse,
    #[serde(rename = "0259")]
    Guyaanse,
    #[serde(rename = "0261")]
    Paraguayaanse,
    #[serde(rename = "0262")]
    Peruaanse,
    #[serde(rename = "0263")]
    Surinaamse,
    #[serde(rename = "0264")]
    Uruguayaanse,
    #[serde(rename = "0265")]
    Venezolaanse,
    #[serde(rename = "0267")]
    Grenadaanse,
    #[serde(rename = "0268")]
    BurgerVanSaintKittsEnNevis,
    #[serde(rename = "0300")]
    Afghaanse,
    #[serde(rename = "0301")]
    Bahreinse,
    #[serde(rename = "0302")]
    Bhutaanse,
    #[serde(rename = "0303")]
    Burmaanse,
    #[serde(rename = "0304")]
    Bruneise,
    #[serde(rename = "0305")]
    Cambodjaanse,
    #[serde(rename = "0306")]
    SriLankaanse,
    #[serde(rename = "0307")]
    Chinese,
    #[serde(rename = "0308")]
    Cyprische,
    #[serde(rename = "0309")]
    Filipijnse,
    #[serde(rename = "0310")]
    Taiwanese,
    #[serde(rename = "0312")]
    Indiase,
    #[serde(rename = "0313")]
    Indonesische,
    #[serde(rename = "0314")]
    Iraakse,
    #[serde(rename = "0315")]
    Iraanse,
    #[serde(rename = "0316")]
    Israelische,
    #[serde(rename = "0317")]
    Japanse,
    #[serde(rename = "0318")]
    NoordJemenitische,
    #[serde(rename = "0319")]
    Jordaanse,
    #[serde(rename = "0320")]
    Koeweitse,
    #[serde(rename = "0321")]
    Laotiaanse,
    #[serde(rename = "0322")]
    Libanese,
    #[serde(rename = "0324")]
    Maldivische,
    #[serde(rename = "0325")]
    Maleisische,
    #[serde(rename = "0326")]
    Mongolische,
    #[serde(rename = "0327")]
    Omaanse,
    #[serde(rename = "0328")]
    Nepalese,
    #[serde(rename = "0329")]
    NoordKoreaanse,
    #[serde(rename = "0331")]
    Pakistaanse,
    #[serde(rename = "0333")]
    Qatarese,
    #[serde(rename = "0334")]
    SaoediArabische,
    #[serde(rename = "0335")]
    Singaporese,
    #[serde(rename = "0336")]
    Syrische,
    #[serde(rename = "0337")]
    Thaise,
    #[serde(rename = "0338")]
    BurgerVanDeVerenigdeArabischeEmiraten,
    #[serde(rename = "0339")]
    Turkse,
    #[serde(rename = "0340")]
    ZuidJemenitische,
    #[serde(rename = "0341")]
    ZuidKoreaanse,
    #[serde(rename = "0342")]
    Vietnamese,
    #[serde(rename = "0345")]
    Bengalese,
    #[serde(rename = "0400")]
    Australische,
    #[serde(rename = "0401")]
    PapoeaNieuwGuinese,
    #[serde(rename = "0402")]
    NieuwZeelandse,
    #[serde(rename = "0404")]
    WestSamoaanse,
    #[serde(rename = "0405")]
    Samoaanse,
    #[serde(rename = "0421")]
    BurgerVanAntiguaEnBarbuda,
    #[serde(rename = "0424")]
    Vanuatuaanse,
    #[serde(rename = "0425")]
    Fijische,
    #[serde(rename = "0429")]
    BurgerVanBritseAfhankelijkeGebieden,
    #[serde(rename = "0430")]
    Tongaanse,
    #[serde(rename = "0431")]
    Nauruaanse,
    #[serde(rename = "0432")]
    Palause,
    #[serde(rename = "0437")]
    AmerikaansOnderdaan,
    #[serde(rename = "0442")]
    Salomonseilandse,
    #[serde(rename = "0443")]
    Micronesische,
    #[serde(rename = "0444")]
    Seychelse,
    #[serde(rename = "0445")]
    Kiribatische,
    #[serde(rename = "0446")]
    Tuvaluaanse,
    #[serde(rename = "0447")]
    SaintLuciaanse,
    #[serde(rename = "0448")]
    BurgerVanDominica,
    #[serde(rename = "0449")]
    BurgerVanSaintVincentEnDeGrenadines,
    #[serde(rename = "0450")]
    BritishNationalOverseas,
    #[serde(rename = "0451")]
    BurgerVanDemocratischeRepubliekCongo,
    #[serde(rename = "0452")]
    BurgerVanTimorLeste,
    #[serde(rename = "0453")]
    BurgerVanServieEnMontenegro,
    #[serde(rename = "0454")]
    Servische,
    #[serde(rename = "0455")]
    Montenegrijnse,
    #[serde(rename = "0499")]
    Staatloos,
    #[serde(rename = "0500")]
    VastgesteldNietNederlander,

    /// Not in the RvIG Nationaliteitentabel
    #[serde(other)]
    Overig,
}

impl RvigNationalityCode {
    /// Maps to an ISO 3166-1 alpha-2 country
    ///
    /// The following variants are mapped:
    /// * Renamed/merged countries (e.g. Burma -> Myanmar) map to its current code.
    /// * Split/dissolved countries have no official ISO 3166-1 code: e.g. "Joegoslavische", "BurgerVanDeSovjetUnie",
    ///   "TsjechoSlowaakse", "BurgerVanServieEnMontenegro"
    /// * "Kosovaarse" has no official ISO 3166-1 code.
    /// * Returns `None` for variants that aren't an ISO 3166-1 country: e.g. "Onbekend", "Staatloos",
    ///   "BehandeldAlsNederlander", "VastgesteldNietNederlander"
    fn alpha2(&self) -> Option<IsoCountryCode> {
        match self {
            Self::Onbekend => None,
            Self::Nederlandse => Some(IsoCountryCode::Netherlands),
            Self::BehandeldAlsNederlander => None,
            Self::Slowaakse => Some(IsoCountryCode::Slovakia),
            Self::Tsjechische => Some(IsoCountryCode::Czechia),
            Self::BurgerVanBosnieHerzegovina => Some(IsoCountryCode::BosniaAndHerzegovina),
            Self::Georgische => Some(IsoCountryCode::Georgia),
            Self::Turkmeense => Some(IsoCountryCode::Turkmenistan),
            Self::Tadzjiekse => Some(IsoCountryCode::Tajikistan),
            Self::Oezbeekse => Some(IsoCountryCode::Uzbekistan),
            Self::Oekrainense => Some(IsoCountryCode::Ukraine),
            Self::Kirgizische => Some(IsoCountryCode::Kyrgyzstan),
            Self::Moldavische => Some(IsoCountryCode::Moldova),
            Self::Kazachse => Some(IsoCountryCode::Kazakhstan),
            Self::Belarussische => Some(IsoCountryCode::Belarus),
            Self::Azerbeidzjaanse => Some(IsoCountryCode::Azerbaijan),
            Self::Armeense => Some(IsoCountryCode::Armenia),
            Self::Russische => Some(IsoCountryCode::Russia),
            Self::Sloveense => Some(IsoCountryCode::Slovenia),
            Self::Kroatische => Some(IsoCountryCode::Croatia),
            Self::Letse => Some(IsoCountryCode::Latvia),
            Self::Estische => Some(IsoCountryCode::Estonia),
            Self::Litouwse => Some(IsoCountryCode::Lithuania),
            Self::Marshalleilandse => Some(IsoCountryCode::MarshallIslands),
            Self::Myanmarese => Some(IsoCountryCode::Myanmar),
            Self::Namibische => Some(IsoCountryCode::Namibia),
            Self::Albanese => Some(IsoCountryCode::Albania),
            Self::Andorrese => Some(IsoCountryCode::Andorra),
            Self::Belgische => Some(IsoCountryCode::Belgium),
            Self::Bulgaarse => Some(IsoCountryCode::Bulgaria),
            Self::Deense => Some(IsoCountryCode::Denmark),
            Self::BurgerVanDeBondsrepubliekDuitsland => Some(IsoCountryCode::Germany),
            Self::Finse => Some(IsoCountryCode::Finland),
            Self::Franse => Some(IsoCountryCode::France),
            Self::Jemenitische => Some(IsoCountryCode::Yemen),
            Self::Griekse => Some(IsoCountryCode::Greece),
            Self::BritsBurger => Some(IsoCountryCode::UnitedKingdom),
            Self::Hongaarse => Some(IsoCountryCode::Hungary),
            Self::Ierse => Some(IsoCountryCode::Ireland),
            Self::IJslandse => Some(IsoCountryCode::Iceland),
            Self::Italiaanse => Some(IsoCountryCode::Italy),
            // Yugoslavia (ended 01-02-2004) split into multiple current countries
            Self::Joegoslavische => None,
            Self::Liechtensteinse => Some(IsoCountryCode::Liechtenstein),
            Self::Luxemburgse => Some(IsoCountryCode::Luxembourg),
            Self::Maltese => Some(IsoCountryCode::Malta),
            Self::Monegaskische => Some(IsoCountryCode::Monaco),
            Self::Noorse => Some(IsoCountryCode::Norway),
            Self::Oostenrijkse => Some(IsoCountryCode::Austria),
            Self::Poolse => Some(IsoCountryCode::Poland),
            Self::Portugese => Some(IsoCountryCode::Portugal),
            Self::Roemeense => Some(IsoCountryCode::Romania),
            // The Soviet Union split into multiple current countries (ended 31-12-1991).
            Self::BurgerVanDeSovjetUnie => None,
            Self::SanMarinese => Some(IsoCountryCode::SanMarino),
            Self::Spaanse => Some(IsoCountryCode::Spain),
            // Czechoslovakia split into Czechia and Slovakia on 01-01-1993.
            Self::TsjechoSlowaakse => None,
            Self::Vaticaanse => Some(IsoCountryCode::HolySee),
            Self::Zweedse => Some(IsoCountryCode::Sweden),
            Self::Zwitserse => Some(IsoCountryCode::Switzerland),
            // Eastern Germany merged into reunified Germany.
            Self::OostDuitse => Some(IsoCountryCode::Germany),
            // A British-nationality-law status.
            Self::BritsOnderdaan => Some(IsoCountryCode::UnitedKingdom),
            Self::Eritrese => Some(IsoCountryCode::Eritrea),
            // A British-nationality-law status.
            Self::BritsOverzeesBurger => Some(IsoCountryCode::UnitedKingdom),
            Self::Macedonische => Some(IsoCountryCode::NorthMacedonia),
            // Kosovo has no official ISO 3166-1 code, see [`Self::as_nationality_attribute`].
            Self::Kosovaarse => None,
            Self::BurgerVanNoordMacedonie => Some(IsoCountryCode::NorthMacedonia),
            Self::Algerijnse => Some(IsoCountryCode::Algeria),
            Self::Angolese => Some(IsoCountryCode::Angola),
            Self::Burundese => Some(IsoCountryCode::Burundi),
            Self::Botswaanse => Some(IsoCountryCode::Botswana),
            Self::Burkinese => Some(IsoCountryCode::BurkinaFaso),
            Self::CentraalAfrikaanse => Some(IsoCountryCode::CentralAfricanRepublic),
            Self::Comorese => Some(IsoCountryCode::Comoros),
            Self::BurgerVanCongo => Some(IsoCountryCode::Congo),
            Self::Beninse => Some(IsoCountryCode::Benin),
            Self::Egyptische => Some(IsoCountryCode::Egypt),
            Self::EquatoriaalGuinese => Some(IsoCountryCode::EquatorialGuinea),
            Self::Ethiopische => Some(IsoCountryCode::Ethiopia),
            Self::Djiboutiaanse => Some(IsoCountryCode::Djibouti),
            Self::Gabonese => Some(IsoCountryCode::Gabon),
            Self::Gambiaanse => Some(IsoCountryCode::Gambia),
            Self::Ghanese => Some(IsoCountryCode::Ghana),
            Self::Guinese => Some(IsoCountryCode::Guinea),
            Self::Ivoriaanse => Some(IsoCountryCode::CoteDIvoire),
            Self::Kaapverdische => Some(IsoCountryCode::CaboVerde),
            Self::Kameroense => Some(IsoCountryCode::Cameroon),
            Self::Kenyaanse => Some(IsoCountryCode::Kenya),
            // Zaire was renamed Democratic Republic of the Congo on 15-07-1997.
            Self::Zairese => Some(IsoCountryCode::DemocraticRepublicOfTheCongo),
            Self::Lesothaanse => Some(IsoCountryCode::Lesotho),
            Self::Liberiaanse => Some(IsoCountryCode::Liberia),
            Self::Libische => Some(IsoCountryCode::Libya),
            Self::Malagassische => Some(IsoCountryCode::Madagascar),
            Self::Malawische => Some(IsoCountryCode::Malawi),
            Self::Malinese => Some(IsoCountryCode::Mali),
            Self::Marokkaanse => Some(IsoCountryCode::Morocco),
            Self::Mauritaanse => Some(IsoCountryCode::Mauritania),
            Self::Mauritiaanse => Some(IsoCountryCode::Mauritius),
            Self::Mozambikaanse => Some(IsoCountryCode::Mozambique),
            // Swaziland was renamed Eswatini on 19-04-2018.
            Self::Swazische => Some(IsoCountryCode::Eswatini),
            Self::Nigerese => Some(IsoCountryCode::Niger),
            Self::Nigeriaanse => Some(IsoCountryCode::Nigeria),
            Self::Ugandese => Some(IsoCountryCode::Uganda),
            Self::GuineeBissause => Some(IsoCountryCode::GuineaBissau),
            Self::ZuidAfrikaanse => Some(IsoCountryCode::SouthAfrica),
            Self::Eswatinische => Some(IsoCountryCode::Eswatini),
            Self::Zimbabwaanse => Some(IsoCountryCode::Zimbabwe),
            Self::Rwandese => Some(IsoCountryCode::Rwanda),
            Self::BurgerVanSaoTomeEnPrincipe => Some(IsoCountryCode::SaoTomeAndPrincipe),
            Self::Senegalese => Some(IsoCountryCode::Senegal),
            Self::SierraLeoonse => Some(IsoCountryCode::SierraLeone),
            Self::Soedanese => Some(IsoCountryCode::Sudan),
            Self::Somalische => Some(IsoCountryCode::Somalia),
            Self::Tanzaniaanse => Some(IsoCountryCode::Tanzania),
            Self::Togolese => Some(IsoCountryCode::Togo),
            Self::Tsjadische => Some(IsoCountryCode::Chad),
            Self::Tunesische => Some(IsoCountryCode::Tunisia),
            Self::Zambiaanse => Some(IsoCountryCode::Zambia),
            Self::ZuidSoedanese => Some(IsoCountryCode::SouthSudan),
            Self::Bahamaanse => Some(IsoCountryCode::Bahamas),
            Self::Belizaanse => Some(IsoCountryCode::Belize),
            Self::Canadese => Some(IsoCountryCode::Canada),
            Self::CostaRicaanse => Some(IsoCountryCode::CostaRica),
            Self::Cubaanse => Some(IsoCountryCode::Cuba),
            Self::Dominicaanse => Some(IsoCountryCode::DominicanRepublic),
            Self::Salvadoraanse => Some(IsoCountryCode::ElSalvador),
            Self::Guatemalaanse => Some(IsoCountryCode::Guatemala),
            Self::Haitiaanse => Some(IsoCountryCode::Haiti),
            Self::Hondurese => Some(IsoCountryCode::Honduras),
            Self::Jamaicaanse => Some(IsoCountryCode::Jamaica),
            Self::Mexicaanse => Some(IsoCountryCode::Mexico),
            Self::Nicaraguaanse => Some(IsoCountryCode::Nicaragua),
            Self::Panamese => Some(IsoCountryCode::Panama),
            Self::BurgerVanTrinidadEnTobago => Some(IsoCountryCode::TrinidadAndTobago),
            Self::AmerikaansBurger => Some(IsoCountryCode::UnitedStates),
            Self::Argentijnse => Some(IsoCountryCode::Argentina),
            Self::Barbadaanse => Some(IsoCountryCode::Barbados),
            Self::Boliviaanse => Some(IsoCountryCode::Bolivia),
            Self::Braziliaanse => Some(IsoCountryCode::Brazil),
            Self::Chileense => Some(IsoCountryCode::Chile),
            Self::Colombiaanse => Some(IsoCountryCode::Colombia),
            Self::Ecuadoraanse => Some(IsoCountryCode::Ecuador),
            Self::Guyaanse => Some(IsoCountryCode::Guyana),
            Self::Paraguayaanse => Some(IsoCountryCode::Paraguay),
            Self::Peruaanse => Some(IsoCountryCode::Peru),
            Self::Surinaamse => Some(IsoCountryCode::Suriname),
            Self::Uruguayaanse => Some(IsoCountryCode::Uruguay),
            Self::Venezolaanse => Some(IsoCountryCode::Venezuela),
            Self::Grenadaanse => Some(IsoCountryCode::Grenada),
            Self::BurgerVanSaintKittsEnNevis => Some(IsoCountryCode::SaintKittsAndNevis),
            Self::Afghaanse => Some(IsoCountryCode::Afghanistan),
            Self::Bahreinse => Some(IsoCountryCode::Bahrain),
            Self::Bhutaanse => Some(IsoCountryCode::Bhutan),
            // Burma was renamed to Myanmar on 18-06-1989.
            Self::Burmaanse => Some(IsoCountryCode::Myanmar),
            Self::Bruneise => Some(IsoCountryCode::BruneiDarussalam),
            Self::Cambodjaanse => Some(IsoCountryCode::Cambodia),
            Self::SriLankaanse => Some(IsoCountryCode::SriLanka),
            Self::Chinese => Some(IsoCountryCode::China),
            Self::Cyprische => Some(IsoCountryCode::Cyprus),
            Self::Filipijnse => Some(IsoCountryCode::Philippines),
            Self::Taiwanese => Some(IsoCountryCode::Taiwan),
            Self::Indiase => Some(IsoCountryCode::India),
            Self::Indonesische => Some(IsoCountryCode::Indonesia),
            Self::Iraakse => Some(IsoCountryCode::Iraq),
            Self::Iraanse => Some(IsoCountryCode::Iran),
            Self::Israelische => Some(IsoCountryCode::Israel),
            Self::Japanse => Some(IsoCountryCode::Japan),
            // North Yemen merged into Yemen on 22-05-1990.
            Self::NoordJemenitische => Some(IsoCountryCode::Yemen),
            Self::Jordaanse => Some(IsoCountryCode::Jordan),
            Self::Koeweitse => Some(IsoCountryCode::Kuwait),
            Self::Laotiaanse => Some(IsoCountryCode::Laos),
            Self::Libanese => Some(IsoCountryCode::Lebanon),
            Self::Maldivische => Some(IsoCountryCode::Maldives),
            Self::Maleisische => Some(IsoCountryCode::Malaysia),
            Self::Mongolische => Some(IsoCountryCode::Mongolia),
            Self::Omaanse => Some(IsoCountryCode::Oman),
            Self::Nepalese => Some(IsoCountryCode::Nepal),
            Self::NoordKoreaanse => Some(IsoCountryCode::NorthKorea),
            Self::Pakistaanse => Some(IsoCountryCode::Pakistan),
            Self::Qatarese => Some(IsoCountryCode::Qatar),
            Self::SaoediArabische => Some(IsoCountryCode::SaudiArabia),
            Self::Singaporese => Some(IsoCountryCode::Singapore),
            Self::Syrische => Some(IsoCountryCode::Syria),
            Self::Thaise => Some(IsoCountryCode::Thailand),
            Self::BurgerVanDeVerenigdeArabischeEmiraten => Some(IsoCountryCode::UnitedArabEmirates),
            Self::Turkse => Some(IsoCountryCode::Turkiye),
            // South Yemen merged into Yemen on 22-05-1990.
            Self::ZuidJemenitische => Some(IsoCountryCode::Yemen),
            Self::ZuidKoreaanse => Some(IsoCountryCode::SouthKorea),
            Self::Vietnamese => Some(IsoCountryCode::Vietnam),
            Self::Bengalese => Some(IsoCountryCode::Bangladesh),
            Self::Australische => Some(IsoCountryCode::Australia),
            Self::PapoeaNieuwGuinese => Some(IsoCountryCode::PapuaNewGuinea),
            Self::NieuwZeelandse => Some(IsoCountryCode::NewZealand),
            // Western Samoa was renamed Samoa on 04-07-1997.
            Self::WestSamoaanse => Some(IsoCountryCode::Samoa),
            Self::Samoaanse => Some(IsoCountryCode::Samoa),
            Self::BurgerVanAntiguaEnBarbuda => Some(IsoCountryCode::AntiguaAndBarbuda),
            Self::Vanuatuaanse => Some(IsoCountryCode::Vanuatu),
            Self::Fijische => Some(IsoCountryCode::Fiji),
            // A British-nationality-law status.
            Self::BurgerVanBritseAfhankelijkeGebieden => Some(IsoCountryCode::UnitedKingdom),
            Self::Tongaanse => Some(IsoCountryCode::Tonga),
            Self::Nauruaanse => Some(IsoCountryCode::Nauru),
            Self::Palause => Some(IsoCountryCode::Palau),
            // A US-nationality-law status.
            Self::AmerikaansOnderdaan => Some(IsoCountryCode::UnitedStates),
            Self::Salomonseilandse => Some(IsoCountryCode::SolomonIslands),
            Self::Micronesische => Some(IsoCountryCode::Micronesia),
            Self::Seychelse => Some(IsoCountryCode::Seychelles),
            Self::Kiribatische => Some(IsoCountryCode::Kiribati),
            Self::Tuvaluaanse => Some(IsoCountryCode::Tuvalu),
            Self::SaintLuciaanse => Some(IsoCountryCode::SaintLucia),
            Self::BurgerVanDominica => Some(IsoCountryCode::Dominica),
            Self::BurgerVanSaintVincentEnDeGrenadines => Some(IsoCountryCode::SaintVincentAndTheGrenadines),
            // A British-nationality-law status.
            Self::BritishNationalOverseas => Some(IsoCountryCode::UnitedKingdom),
            Self::BurgerVanDemocratischeRepubliekCongo => Some(IsoCountryCode::DemocraticRepublicOfTheCongo),
            Self::BurgerVanTimorLeste => Some(IsoCountryCode::TimorLeste),
            // Serbia and Montenegro split into Serbia and Montenegro on 03-06-2006.
            Self::BurgerVanServieEnMontenegro => None,
            Self::Servische => Some(IsoCountryCode::Serbia),
            Self::Montenegrijnse => Some(IsoCountryCode::Montenegro),
            Self::Staatloos => None,
            Self::VastgesteldNietNederlander => None,
            Self::Overig => None,
        }
    }
}

impl From<RvigNationalityCode> for RulebookNationality {
    fn from(value: RvigNationalityCode) -> Self {
        match value.alpha2() {
            Some(country) => Self::Iso(country),
            None if matches!(value, RvigNationalityCode::Kosovaarse) => Self::Other("XK".to_owned()),
            None if matches!(value, RvigNationalityCode::Staatloos) => Self::Stateless,
            None => Self::Unknown,
        }
    }
}

/// RvIG's Tabel 34 Landentabel codes, used for both birth country and residence country. Source:
/// <https://publicaties.rvig.nl/media/13285/download>. `Overig` is a `serde(other)` catch-all for any future RvIG code
/// not yet in this enum.
#[derive(Clone, Deserialize)]
pub enum RvigCountryCode {
    #[serde(rename = "0000")]
    Onbekend,
    #[serde(rename = "5001")]
    Canada,
    #[serde(rename = "5002")]
    Frankrijk,
    #[serde(rename = "5003")]
    Zwitserland,
    #[serde(rename = "5004")]
    Rhodesie,
    #[serde(rename = "5005")]
    Malawi,
    #[serde(rename = "5006")]
    Cuba,
    #[serde(rename = "5007")]
    Suriname,
    #[serde(rename = "5008")]
    Tunesie,
    #[serde(rename = "5009")]
    Oostenrijk,
    #[serde(rename = "5010")]
    Belgie,
    #[serde(rename = "5011")]
    Botswana,
    #[serde(rename = "5012")]
    Iran,
    #[serde(rename = "5013")]
    NieuwZeeland,
    #[serde(rename = "5014")]
    ZuidAfrika,
    #[serde(rename = "5015")]
    Denemarken,
    #[serde(rename = "5016")]
    NoordJemen,
    #[serde(rename = "5017")]
    Hongarije,
    #[serde(rename = "5018")]
    SaoediArabie,
    #[serde(rename = "5019")]
    Liberia,
    #[serde(rename = "5020")]
    Ethiopie,
    #[serde(rename = "5021")]
    Chili,
    #[serde(rename = "5022")]
    Marokko,
    #[serde(rename = "5023")]
    Togo,
    #[serde(rename = "5024")]
    Ghana,
    #[serde(rename = "5025")]
    Laos,
    #[serde(rename = "5026")]
    Angola,
    #[serde(rename = "5027")]
    Filipijnen,
    #[serde(rename = "5028")]
    Zambia,
    #[serde(rename = "5029")]
    Mali,
    #[serde(rename = "5030")]
    Ivoorkust,
    #[serde(rename = "5031")]
    Birma,
    #[serde(rename = "5032")]
    Monaco,
    #[serde(rename = "5033")]
    Colombia,
    #[serde(rename = "5034")]
    Albanie,
    #[serde(rename = "5035")]
    Kameroen,
    #[serde(rename = "5036")]
    ZuidVietnam,
    #[serde(rename = "5037")]
    Singapore,
    #[serde(rename = "5038")]
    Paraguay,
    #[serde(rename = "5039")]
    Zweden,
    #[serde(rename = "5040")]
    Cyprus,
    #[serde(rename = "5041")]
    AustralischNieuwGuinea,
    #[serde(rename = "5042")]
    Brunei,
    #[serde(rename = "5043")]
    Irak,
    #[serde(rename = "5044")]
    Mauritius,
    #[serde(rename = "5045")]
    Vaticaanstad,
    #[serde(rename = "5046")]
    Kashmir,
    #[serde(rename = "5047")]
    Myanmar,
    #[serde(rename = "5048")]
    Jemen,
    #[serde(rename = "5049")]
    Slovenie,
    #[serde(rename = "5050")]
    Zaire,
    #[serde(rename = "5051")]
    Kroatie,
    #[serde(rename = "5052")]
    Taiwan,
    #[serde(rename = "5053")]
    Rusland,
    #[serde(rename = "5054")]
    Armenie,
    #[serde(rename = "5055")]
    Ascension,
    #[serde(rename = "5056")]
    Azoren,
    #[serde(rename = "5057")]
    Bahrein,
    #[serde(rename = "5058")]
    Bhutan,
    #[serde(rename = "5059")]
    BritseAntillen,
    #[serde(rename = "5060")]
    Comoren,
    #[serde(rename = "5061")]
    Falklandeilanden,
    #[serde(rename = "5062")]
    FransGuyana,
    #[serde(rename = "5063")]
    FransSomaliland,
    #[serde(rename = "5064")]
    GilbertEnEllice,
    #[serde(rename = "5065")]
    Groenland,
    #[serde(rename = "5066")]
    Guadeloupe,
    #[serde(rename = "5067")]
    KaapverdischeEilanden,
    #[serde(rename = "5068")]
    Macau,
    #[serde(rename = "5069")]
    Martinique,
    #[serde(rename = "5070")]
    Mozambique,
    #[serde(rename = "5071")]
    Pitcairneilanden,
    #[serde(rename = "5072")]
    GuineeBissau,
    #[serde(rename = "5073")]
    Reunion,
    #[serde(rename = "5074")]
    SaintPierreEnMiquelon,
    #[serde(rename = "5075")]
    SeychellenEnAmiranten,
    #[serde(rename = "5076")]
    Tonga,
    #[serde(rename = "5077")]
    WallisEnFutuna,
    #[serde(rename = "5078")]
    ZuidwestAfrika,
    #[serde(rename = "5079")]
    FransIndie,
    #[serde(rename = "5080")]
    Johnston,
    #[serde(rename = "5081")]
    Kedah,
    #[serde(rename = "5082")]
    Kelantan,
    #[serde(rename = "5083")]
    Malakka,
    #[serde(rename = "5084")]
    Mayotte,
    #[serde(rename = "5085")]
    NegriSembilan,
    #[serde(rename = "5086")]
    Pahang,
    #[serde(rename = "5087")]
    Perak,
    #[serde(rename = "5088")]
    Perlis,
    #[serde(rename = "5089")]
    PortugeesIndie,
    #[serde(rename = "5090")]
    Selangor,
    #[serde(rename = "5091")]
    Sikkim,
    #[serde(rename = "5092")]
    SaintVincentEnDeGrenadines,
    #[serde(rename = "5093")]
    Spitsbergen,
    #[serde(rename = "5094")]
    Trengganu,
    #[serde(rename = "5095")]
    Aruba,
    #[serde(rename = "5096")]
    BurkinaFaso,
    #[serde(rename = "5097")]
    Azerbeidzjan,
    #[serde(rename = "5098")]
    Belarus,
    #[serde(rename = "5099")]
    Kazachstan,
    #[serde(rename = "5100")]
    Macedonie,
    #[serde(rename = "5101")]
    OostTimor,
    #[serde(rename = "5102")]
    ServieEnMontenegro,
    #[serde(rename = "5103")]
    Servie,
    #[serde(rename = "5104")]
    Montenegro,
    #[serde(rename = "5105")]
    Kosovo,
    #[serde(rename = "5106")]
    Bonaire,
    #[serde(rename = "5107")]
    Curacao,
    #[serde(rename = "5108")]
    Saba,
    #[serde(rename = "5109")]
    SintEustatius,
    #[serde(rename = "5110")]
    SintMaarten,
    #[serde(rename = "5111")]
    ZuidSoedan,
    #[serde(rename = "5112")]
    GazastrookEnWestelijkeJordaanoever,
    #[serde(rename = "5113")]
    RepubliekNoordMacedonie,
    #[serde(rename = "6000")]
    Moldavie,
    #[serde(rename = "6001")]
    Burundi,
    #[serde(rename = "6002")]
    Finland,
    #[serde(rename = "6003")]
    Griekenland,
    #[serde(rename = "6004")]
    Guatemala,
    #[serde(rename = "6005")]
    Nigeria,
    #[serde(rename = "6006")]
    Libie,
    #[serde(rename = "6007")]
    Ierland,
    #[serde(rename = "6008")]
    Brazilie,
    #[serde(rename = "6009")]
    Rwanda,
    #[serde(rename = "6010")]
    Venezuela,
    #[serde(rename = "6011")]
    IJsland,
    #[serde(rename = "6012")]
    Liechtenstein,
    #[serde(rename = "6013")]
    Somalie,
    #[serde(rename = "6014")]
    VerenigdeStatenVanAmerika,
    #[serde(rename = "6015")]
    Bolivia,
    #[serde(rename = "6016")]
    Australie,
    #[serde(rename = "6017")]
    Jamaica,
    #[serde(rename = "6018")]
    Luxemburg,
    #[serde(rename = "6019")]
    Tsjaad,
    #[serde(rename = "6020")]
    Mauritanie,
    #[serde(rename = "6021")]
    Kirgizie,
    #[serde(rename = "6022")]
    China,
    #[serde(rename = "6023")]
    Afghanistan,
    #[serde(rename = "6024")]
    Indonesie,
    #[serde(rename = "6025")]
    Guyana,
    #[serde(rename = "6026")]
    NoordVietnam,
    #[serde(rename = "6027")]
    Noorwegen,
    #[serde(rename = "6028")]
    SanMarino,
    #[serde(rename = "6029")]
    Duitsland,
    #[serde(rename = "6030")]
    Nederland,
    #[serde(rename = "6031")]
    Cambodja,
    #[serde(rename = "6032")]
    Fiji,
    #[serde(rename = "6033")]
    Bahamas,
    #[serde(rename = "6034")]
    Israel,
    #[serde(rename = "6035")]
    Nepal,
    #[serde(rename = "6036")]
    ZuidKorea,
    #[serde(rename = "6037")]
    Spanje,
    #[serde(rename = "6038")]
    Oekraine,
    #[serde(rename = "6039")]
    VerenigdKoninkrijk,
    #[serde(rename = "6040")]
    Niger,
    #[serde(rename = "6041")]
    Haiti,
    #[serde(rename = "6042")]
    Jordanie,
    #[serde(rename = "6043")]
    Turkije,
    #[serde(rename = "6044")]
    TrinidadEnTobago,
    #[serde(rename = "6045")]
    Joegoslavie,
    #[serde(rename = "6046")]
    OpperVolta,
    #[serde(rename = "6047")]
    Algerije,
    #[serde(rename = "6048")]
    Gabon,
    #[serde(rename = "6049")]
    NoordKorea,
    #[serde(rename = "6050")]
    Oezbekistan,
    #[serde(rename = "6051")]
    SierraLeone,
    #[serde(rename = "6052")]
    BritsHonduras,
    #[serde(rename = "6053")]
    CanarischeEilanden,
    #[serde(rename = "6054")]
    FransPolynesie,
    #[serde(rename = "6055")]
    Gibraltar,
    #[serde(rename = "6056")]
    PortugeesTimor,
    #[serde(rename = "6057")]
    Tadzjikistan,
    #[serde(rename = "6058")]
    BritseSalomonseilanden,
    #[serde(rename = "6059")]
    SaoTomeEnPrincipe,
    #[serde(rename = "6060")]
    SintHelena,
    #[serde(rename = "6061")]
    TristanDaCunha,
    #[serde(rename = "6062")]
    WestSamoa,
    #[serde(rename = "6063")]
    Turkmenistan,
    #[serde(rename = "6064")]
    Georgie,
    #[serde(rename = "6065")]
    BosnieEnHerzegovina,
    #[serde(rename = "6066")]
    Tsjechie,
    #[serde(rename = "6067")]
    Slowakije,
    #[serde(rename = "6068")]
    FederaleRepubliekJoegoslavie,
    #[serde(rename = "6069")]
    DemocratischeRepubliekCongo,
    #[serde(rename = "7001")]
    Oeganda,
    #[serde(rename = "7002")]
    Kenia,
    #[serde(rename = "7003")]
    Malta,
    #[serde(rename = "7004")]
    Barbados,
    #[serde(rename = "7005")]
    Andorra,
    #[serde(rename = "7006")]
    Mexico,
    #[serde(rename = "7007")]
    CostaRica,
    #[serde(rename = "7008")]
    Gambia,
    #[serde(rename = "7009")]
    Syrie,
    #[serde(rename = "7011")]
    NederlandseAntillen,
    #[serde(rename = "7012")]
    ZuidJemen,
    #[serde(rename = "7014")]
    Egypte,
    #[serde(rename = "7015")]
    Argentinie,
    #[serde(rename = "7016")]
    Lesotho,
    #[serde(rename = "7017")]
    Honduras,
    #[serde(rename = "7018")]
    Nicaragua,
    #[serde(rename = "7020")]
    Pakistan,
    #[serde(rename = "7021")]
    Senegal,
    #[serde(rename = "7023")]
    Dahomey,
    #[serde(rename = "7024")]
    Bulgarije,
    #[serde(rename = "7026")]
    Maleisie,
    #[serde(rename = "7027")]
    DominicaanseRepubliek,
    #[serde(rename = "7028")]
    Polen,
    #[serde(rename = "7029")]
    RuslandOud,
    #[serde(rename = "7030")]
    BritseMaagdeneilanden,
    #[serde(rename = "7031")]
    Tanzania,
    #[serde(rename = "7032")]
    ElSalvador,
    #[serde(rename = "7033")]
    SriLanka,
    #[serde(rename = "7034")]
    Soedan,
    #[serde(rename = "7035")]
    Japan,
    #[serde(rename = "7036")]
    Hongkong,
    #[serde(rename = "7037")]
    Panama,
    #[serde(rename = "7038")]
    Uruguay,
    #[serde(rename = "7039")]
    Ecuador,
    #[serde(rename = "7040")]
    Guinee,
    #[serde(rename = "7041")]
    Maldiven,
    #[serde(rename = "7042")]
    Thailand,
    #[serde(rename = "7043")]
    Libanon,
    #[serde(rename = "7044")]
    Italie,
    #[serde(rename = "7045")]
    Koeweit,
    #[serde(rename = "7046")]
    India,
    #[serde(rename = "7047")]
    Roemenie,
    #[serde(rename = "7048")]
    TsjechoSlowakije,
    #[serde(rename = "7049")]
    Peru,
    #[serde(rename = "7050")]
    Portugal,
    #[serde(rename = "7051")]
    Oman,
    #[serde(rename = "7052")]
    Mongolie,
    #[serde(rename = "7053")]
    Samoa,
    #[serde(rename = "7054")]
    VerenigdeArabischeEmiraten,
    #[serde(rename = "7055")]
    Tibet,
    #[serde(rename = "7057")]
    Nauru,
    #[serde(rename = "7058")]
    NederlandsNieuwGuinea,
    #[serde(rename = "7059")]
    Tanganyika,
    #[serde(rename = "7060")]
    Palestina,
    #[serde(rename = "7062")]
    BritsWestIndie,
    #[serde(rename = "7063")]
    PortugeesAfrika,
    #[serde(rename = "7064")]
    Letland,
    #[serde(rename = "7065")]
    Estland,
    #[serde(rename = "7066")]
    Litouwen,
    #[serde(rename = "7067")]
    BritsAfrika,
    #[serde(rename = "7068")]
    BelgischCongo,
    #[serde(rename = "7070")]
    BritsIndie,
    #[serde(rename = "7071")]
    NoordRhodesie,
    #[serde(rename = "7072")]
    ZuidRhodesie,
    #[serde(rename = "7073")]
    Saarland,
    #[serde(rename = "7074")]
    FransIndochina,
    #[serde(rename = "7075")]
    BritsWestBorneo,
    #[serde(rename = "7076")]
    Goudkust,
    #[serde(rename = "7077")]
    RasAlKhaimah,
    #[serde(rename = "7079")]
    FransCongo,
    #[serde(rename = "7080")]
    Siam,
    #[serde(rename = "7082")]
    BritsOostAfrika,
    #[serde(rename = "7083")]
    BritsNoordBorneo,
    #[serde(rename = "7084")]
    Bangladesh,
    #[serde(rename = "7085")]
    DuitseDemocratischeRepubliek,
    #[serde(rename = "7087")]
    Madeira,
    #[serde(rename = "7088")]
    AmerikaanseMaagdeneilanden,
    #[serde(rename = "7089")]
    AustralischeSalomonseilanden,
    #[serde(rename = "7091")]
    SpaanseSahara,
    #[serde(rename = "7092")]
    Kaaimaneilanden,
    #[serde(rename = "7093")]
    Caicoseilanden,
    #[serde(rename = "7094")]
    Turkseilanden,
    #[serde(rename = "7095")]
    BritsAntarctischTerritorium,
    #[serde(rename = "7096")]
    BritsIndischeOceaanterritorium,
    #[serde(rename = "7097")]
    Cookeilanden,
    #[serde(rename = "7098")]
    Tokelau,
    #[serde(rename = "7099")]
    NieuwCaledonie,
    #[serde(rename = "8000")]
    HawaiiEilanden,
    #[serde(rename = "8001")]
    Guam,
    #[serde(rename = "8002")]
    AmerikaansSamoa,
    #[serde(rename = "8003")]
    Midway,
    #[serde(rename = "8004")]
    RiukiuEilanden,
    #[serde(rename = "8005")]
    Wake,
    #[serde(rename = "8006")]
    PacificEilanden,
    #[serde(rename = "8008")]
    Grenada,
    #[serde(rename = "8009")]
    NoordelijkeMarianen,
    #[serde(rename = "8010")]
    Cabinda,
    #[serde(rename = "8011")]
    CantonEnEnderbury,
    #[serde(rename = "8012")]
    Christmaseiland,
    #[serde(rename = "8013")]
    Cocoseilanden,
    #[serde(rename = "8014")]
    Faeroer,
    #[serde(rename = "8015")]
    Montserrat,
    #[serde(rename = "8016")]
    Norfolk,
    #[serde(rename = "8017")]
    Belize,
    #[serde(rename = "8018")]
    Tasmanie,
    #[serde(rename = "8019")]
    TurksEnCaicoseilanden,
    #[serde(rename = "8020")]
    PuertoRico,
    #[serde(rename = "8021")]
    PapoeaNieuwGuinea,
    #[serde(rename = "8022")]
    Salomonseilanden,
    #[serde(rename = "8023")]
    Benin,
    #[serde(rename = "8024")]
    Vietnam,
    #[serde(rename = "8025")]
    Kaapverdie,
    #[serde(rename = "8026")]
    Seychellen,
    #[serde(rename = "8027")]
    Kiribati,
    #[serde(rename = "8028")]
    Tuvalu,
    #[serde(rename = "8029")]
    SaintLucia,
    #[serde(rename = "8030")]
    Dominica,
    #[serde(rename = "8031")]
    Zimbabwe,
    #[serde(rename = "8032")]
    Dubai,
    #[serde(rename = "8033")]
    NieuweHebriden,
    #[serde(rename = "8034")]
    Kanaaleilanden,
    #[serde(rename = "8035")]
    Man,
    #[serde(rename = "8036")]
    Anguilla,
    #[serde(rename = "8037")]
    SaintKittsEnNevis,
    #[serde(rename = "8038")]
    Antigua,
    #[serde(rename = "8039")]
    SaintVincent,
    #[serde(rename = "8040")]
    Gilberteilanden,
    #[serde(rename = "8041")]
    Panamakanaalzone,
    #[serde(rename = "8042")]
    SaintKittsNevisEnAnguilla,
    #[serde(rename = "8043")]
    Belau,
    #[serde(rename = "8044")]
    Palau,
    #[serde(rename = "8045")]
    AntiguaEnBarbuda,
    #[serde(rename = "8046")]
    SintHelenaAscensionEnTristanDaCunha,
    #[serde(rename = "8047")]
    AmerikaanseKleinereAfgelegenEilanden,
    #[serde(rename = "8048")]
    ZuidGeorgiaEnDeZuidelijkeSandwicheilanden,
    #[serde(rename = "8049")]
    SaintBarthelemy,
    #[serde(rename = "8050")]
    SaintMartin,
    #[serde(rename = "8051")]
    Guernsey,
    #[serde(rename = "8052")]
    Jersey,
    #[serde(rename = "8053")]
    Aland,
    #[serde(rename = "9000")]
    Newfoundland,
    #[serde(rename = "9001")]
    Nyasaland,
    #[serde(rename = "9003")]
    Eritrea,
    #[serde(rename = "9005")]
    Ifni,
    #[serde(rename = "9006")]
    BritsKameroen,
    #[serde(rename = "9007")]
    KeizerWilhelmsland,
    #[serde(rename = "9008")]
    RepubliekCongo,
    #[serde(rename = "9009")]
    CongoKinshasa,
    #[serde(rename = "9010")]
    Madagaskar,
    #[serde(rename = "9013")]
    CongoBrazzaville,
    #[serde(rename = "9014")]
    Leewardeilanden,
    #[serde(rename = "9015")]
    Windwardeilanden,
    #[serde(rename = "9016")]
    FransTerritoriumVoorAfarsEnIssas,
    #[serde(rename = "9017")]
    Phoenixeilanden,
    #[serde(rename = "9020")]
    PortugeesGuinee,
    #[serde(rename = "9022")]
    DuitsZuidwestAfrika,
    #[serde(rename = "9023")]
    Namibie,
    #[serde(rename = "9027")]
    BritsSomaliland,
    #[serde(rename = "9028")]
    ItaliaansSomaliland,
    #[serde(rename = "9030")]
    NederlandsIndie,
    #[serde(rename = "9031")]
    BritsGuyana,
    #[serde(rename = "9036")]
    Swaziland,
    #[serde(rename = "9037")]
    Qatar,
    #[serde(rename = "9038")]
    Eswatini,
    #[serde(rename = "9041")]
    Aden,
    #[serde(rename = "9042")]
    ZuidArabischeFederatie,
    #[serde(rename = "9043")]
    EquatoriaalGuinea,
    #[serde(rename = "9044")]
    SpaansGuinee,
    #[serde(rename = "9047")]
    VerenigdeArabischeRepubliek,
    #[serde(rename = "9048")]
    Bermuda,
    #[serde(rename = "9049")]
    SovjetUnie,
    #[serde(rename = "9050")]
    DuitsOostAfrika,
    #[serde(rename = "9051")]
    Zanzibar,
    #[serde(rename = "9052")]
    Ceylon,
    #[serde(rename = "9053")]
    MuscatEnOman,
    #[serde(rename = "9054")]
    TrucialOman,
    #[serde(rename = "9055")]
    Indochina,
    #[serde(rename = "9056")]
    Marshalleilanden,
    #[serde(rename = "9057")]
    Sarawak,
    #[serde(rename = "9058")]
    BritsBorneo,
    #[serde(rename = "9060")]
    Sabah,
    #[serde(rename = "9061")]
    AbuDhabi,
    #[serde(rename = "9062")]
    Ajman,
    #[serde(rename = "9063")]
    Basutoland,
    #[serde(rename = "9064")]
    Bechuanaland,
    #[serde(rename = "9065")]
    Fujairah,
    #[serde(rename = "9066")]
    FransKameroen,
    #[serde(rename = "9067")]
    Johore,
    #[serde(rename = "9068")]
    Korea,
    #[serde(rename = "9069")]
    Labuan,
    #[serde(rename = "9070")]
    UmmAlQaiwain,
    #[serde(rename = "9071")]
    OostenrijkHongarije,
    #[serde(rename = "9072")]
    PortugeesOostAfrika,
    #[serde(rename = "9073")]
    PortugeesWestAfrika,
    #[serde(rename = "9074")]
    Sharjah,
    #[serde(rename = "9075")]
    StraitsSettlements,
    #[serde(rename = "9076")]
    Abessinie,
    #[serde(rename = "9077")]
    FransWestAfrika,
    #[serde(rename = "9078")]
    FransEquatoriaalAfrika,
    #[serde(rename = "9081")]
    Urundi,
    #[serde(rename = "9082")]
    RuandaUrundi,
    #[serde(rename = "9084")]
    Goa,
    #[serde(rename = "9085")]
    Dantzig,
    #[serde(rename = "9086")]
    CentraalAfrikaanseRepubliek,
    #[serde(rename = "9087")]
    Djibouti,
    #[serde(rename = "9088")]
    Transjordanie,
    #[serde(rename = "9089")]
    BondsrepubliekDuitsland,
    #[serde(rename = "9090")]
    Vanuatu,
    #[serde(rename = "9091")]
    Niue,
    #[serde(rename = "9092")]
    SpaansNoordAfrika,
    #[serde(rename = "9093")]
    WestelijkeSahara,
    #[serde(rename = "9094")]
    Micronesie,
    #[serde(rename = "9095")]
    Svalbardeilanden,
    #[serde(rename = "9999")]
    InternationaalGebied,
    #[serde(other)]
    Overig,
}

impl RvigCountryCode {
    /// Maps to an ISO 3166-1 alpha-2 country code.
    ///
    /// The following variants are mapped:
    /// * Renamed/merged countries (e.g. Rhodesia -> Zimbabwe) map to its current code.
    /// * Split/dissolved countries have no official ISO 3166-1 code: e.g. "Joegoslavie", "SovjetUnie",
    ///   "OostenrijkHongarije"
    /// * "Kosovo" and "Tibet" have no official ISO 3166-1 code.
    fn alpha2(&self) -> Option<IsoCountryCode> {
        match self {
            Self::Onbekend => None,
            Self::Canada => Some(IsoCountryCode::Canada),
            Self::Frankrijk => Some(IsoCountryCode::France),
            Self::Zwitserland => Some(IsoCountryCode::Switzerland),
            // Rhodesia (ended 01-06-1979) became Zimbabwe upon independence in 1980.
            Self::Rhodesie => Some(IsoCountryCode::Zimbabwe),
            Self::Malawi => Some(IsoCountryCode::Malawi),
            Self::Cuba => Some(IsoCountryCode::Cuba),
            Self::Suriname => Some(IsoCountryCode::Suriname),
            Self::Tunesie => Some(IsoCountryCode::Tunisia),
            Self::Oostenrijk => Some(IsoCountryCode::Austria),
            Self::Belgie => Some(IsoCountryCode::Belgium),
            Self::Botswana => Some(IsoCountryCode::Botswana),
            Self::Iran => Some(IsoCountryCode::Iran),
            Self::NieuwZeeland => Some(IsoCountryCode::NewZealand),
            Self::ZuidAfrika => Some(IsoCountryCode::SouthAfrica),
            Self::Denemarken => Some(IsoCountryCode::Denmark),
            // North Yemen merged into Yemen on 22-05-1990.
            Self::NoordJemen => Some(IsoCountryCode::Yemen),
            Self::Hongarije => Some(IsoCountryCode::Hungary),
            Self::SaoediArabie => Some(IsoCountryCode::SaudiArabia),
            Self::Liberia => Some(IsoCountryCode::Liberia),
            Self::Ethiopie => Some(IsoCountryCode::Ethiopia),
            Self::Chili => Some(IsoCountryCode::Chile),
            Self::Marokko => Some(IsoCountryCode::Morocco),
            Self::Togo => Some(IsoCountryCode::Togo),
            Self::Ghana => Some(IsoCountryCode::Ghana),
            Self::Laos => Some(IsoCountryCode::Laos),
            Self::Angola => Some(IsoCountryCode::Angola),
            Self::Filipijnen => Some(IsoCountryCode::Philippines),
            Self::Zambia => Some(IsoCountryCode::Zambia),
            Self::Mali => Some(IsoCountryCode::Mali),
            Self::Ivoorkust => Some(IsoCountryCode::CoteDIvoire),
            // Burma was renamed Myanmar on 18-06-1989.
            Self::Birma => Some(IsoCountryCode::Myanmar),
            Self::Monaco => Some(IsoCountryCode::Monaco),
            Self::Colombia => Some(IsoCountryCode::Colombia),
            Self::Albanie => Some(IsoCountryCode::Albania),
            Self::Kameroen => Some(IsoCountryCode::Cameroon),
            // South Vietnam merged into Vietnam on 02-07-1976.
            Self::ZuidVietnam => Some(IsoCountryCode::Vietnam),
            Self::Singapore => Some(IsoCountryCode::Singapore),
            Self::Paraguay => Some(IsoCountryCode::Paraguay),
            Self::Zweden => Some(IsoCountryCode::Sweden),
            Self::Cyprus => Some(IsoCountryCode::Cyprus),
            // Australian New Guinea became part of Papua New Guinea on 16-09-1975.
            Self::AustralischNieuwGuinea => Some(IsoCountryCode::PapuaNewGuinea),
            Self::Brunei => Some(IsoCountryCode::BruneiDarussalam),
            Self::Irak => Some(IsoCountryCode::Iraq),
            Self::Mauritius => Some(IsoCountryCode::Mauritius),
            Self::Vaticaanstad => Some(IsoCountryCode::HolySee),
            // Kashmir is disputed territory with no official ISO 3166-1 code.
            Self::Kashmir => None,
            Self::Myanmar => Some(IsoCountryCode::Myanmar),
            Self::Jemen => Some(IsoCountryCode::Yemen),
            Self::Slovenie => Some(IsoCountryCode::Slovenia),
            // Zaire was renamed Democratic Republic of the Congo on 15-07-1997 (RvIG lists the withdrawn ISO code "ZR";
            // not adopted here since IsoCountryCode only models current assignments).
            Self::Zaire => Some(IsoCountryCode::DemocraticRepublicOfTheCongo),
            Self::Kroatie => Some(IsoCountryCode::Croatia),
            Self::Taiwan => Some(IsoCountryCode::Taiwan),
            Self::Rusland => Some(IsoCountryCode::Russia),
            Self::Armenie => Some(IsoCountryCode::Armenia),
            // Ascension is part of the single territory Saint Helena, Ascension and Tristan da Cunha.
            Self::Ascension => Some(IsoCountryCode::SaintHelenaAscensionAndTristanDaCunha),
            // The Azores are an autonomous region of Portugal, not separately coded in ISO 3166-1.
            Self::Azoren => Some(IsoCountryCode::Portugal),
            Self::Bahrein => Some(IsoCountryCode::Bahrain),
            Self::Bhutan => Some(IsoCountryCode::Bhutan),
            // The British West Indies devolved into multiple current Caribbean countries; no single successor.
            Self::BritseAntillen => None,
            Self::Comoren => Some(IsoCountryCode::Comoros),
            Self::Falklandeilanden => Some(IsoCountryCode::FalklandIslands),
            Self::FransGuyana => Some(IsoCountryCode::FrenchGuiana),
            // French Somaliland was renamed Djibouti upon independence.
            Self::FransSomaliland => Some(IsoCountryCode::Djibouti),
            // The Gilbert and Ellice Islands split into Kiribati and Tuvalu; no single successor.
            Self::GilbertEnEllice => None,
            Self::Groenland => Some(IsoCountryCode::Greenland),
            Self::Guadeloupe => Some(IsoCountryCode::Guadeloupe),
            // Older code for Cape Verde, superseded by code "8025".
            Self::KaapverdischeEilanden => Some(IsoCountryCode::CaboVerde),
            Self::Macau => Some(IsoCountryCode::Macao),
            Self::Martinique => Some(IsoCountryCode::Martinique),
            Self::Mozambique => Some(IsoCountryCode::Mozambique),
            Self::Pitcairneilanden => Some(IsoCountryCode::Pitcairn),
            Self::GuineeBissau => Some(IsoCountryCode::GuineaBissau),
            Self::Reunion => Some(IsoCountryCode::Reunion),
            Self::SaintPierreEnMiquelon => Some(IsoCountryCode::SaintPierreAndMiquelon),
            // Older code for Seychelles.
            Self::SeychellenEnAmiranten => Some(IsoCountryCode::Seychelles),
            Self::Tonga => Some(IsoCountryCode::Tonga),
            Self::WallisEnFutuna => Some(IsoCountryCode::WallisAndFutuna),
            // South West Africa was renamed Namibia on 21-03-1990.
            Self::ZuidwestAfrika => Some(IsoCountryCode::Namibia),
            // French India's territories were ceded to and incorporated into India.
            Self::FransIndie => Some(IsoCountryCode::India),
            // Johnston Atoll is one of the United States Minor Outlying Islands.
            Self::Johnston => Some(IsoCountryCode::UnitedStatesMinorOutlyingIslands),
            // Pre-federation Malay state, now part of Malaysia.
            Self::Kedah => Some(IsoCountryCode::Malaysia),
            // Pre-federation Malay state, now part of Malaysia.
            Self::Kelantan => Some(IsoCountryCode::Malaysia),
            // Malacca, pre-federation territory, now part of Malaysia.
            Self::Malakka => Some(IsoCountryCode::Malaysia),
            Self::Mayotte => Some(IsoCountryCode::Mayotte),
            // Pre-federation Malay state, now part of Malaysia.
            Self::NegriSembilan => Some(IsoCountryCode::Malaysia),
            // Pre-federation Malay state, now part of Malaysia.
            Self::Pahang => Some(IsoCountryCode::Malaysia),
            // Pre-federation Malay state, now part of Malaysia.
            Self::Perak => Some(IsoCountryCode::Malaysia),
            // Pre-federation Malay state, now part of Malaysia.
            Self::Perlis => Some(IsoCountryCode::Malaysia),
            // Portuguese India (Goa etc.) was annexed by and incorporated into India in 1961.
            Self::PortugeesIndie => Some(IsoCountryCode::India),
            // Pre-federation Malay state, now part of Malaysia.
            Self::Selangor => Some(IsoCountryCode::Malaysia),
            // Sikkim merged with India in 1975.
            Self::Sikkim => Some(IsoCountryCode::India),
            Self::SaintVincentEnDeGrenadines => Some(IsoCountryCode::SaintVincentAndTheGrenadines),
            Self::Spitsbergen => Some(IsoCountryCode::SvalbardAndJanMayen),
            // Pre-federation Malay state, now part of Malaysia.
            Self::Trengganu => Some(IsoCountryCode::Malaysia),
            Self::Aruba => Some(IsoCountryCode::Aruba),
            Self::BurkinaFaso => Some(IsoCountryCode::BurkinaFaso),
            Self::Azerbeidzjan => Some(IsoCountryCode::Azerbaijan),
            Self::Belarus => Some(IsoCountryCode::Belarus),
            Self::Kazachstan => Some(IsoCountryCode::Kazakhstan),
            Self::Macedonie => Some(IsoCountryCode::NorthMacedonia),
            Self::OostTimor => Some(IsoCountryCode::TimorLeste),
            // Serbia and Montenegro split into Serbia and Montenegro in 2006; no single successor (RvIG lists the
            // withdrawn ISO code "CS"; not adopted here).
            Self::ServieEnMontenegro => None,
            Self::Servie => Some(IsoCountryCode::Serbia),
            Self::Montenegro => Some(IsoCountryCode::Montenegro),
            // Kosovo has no official ISO 3166-1 code; deferred rather than adopting the unofficial "XK" placeholder
            // without an explicit product decision.
            Self::Kosovo => None,
            // Bonaire is one of the special municipalities covered by the single ISO code "BQ" (Bonaire, Sint Eustatius
            // and Saba).
            Self::Bonaire => Some(IsoCountryCode::BonaireSintEustatiusAndSaba),
            Self::Curacao => Some(IsoCountryCode::Curacao),
            // Saba is one of the special municipalities covered by the single ISO code "BQ".
            Self::Saba => Some(IsoCountryCode::BonaireSintEustatiusAndSaba),
            // Sint Eustatius is one of the special municipalities covered by the single ISO code "BQ".
            Self::SintEustatius => Some(IsoCountryCode::BonaireSintEustatiusAndSaba),
            Self::SintMaarten => Some(IsoCountryCode::SintMaarten),
            Self::ZuidSoedan => Some(IsoCountryCode::SouthSudan),
            Self::GazastrookEnWestelijkeJordaanoever => Some(IsoCountryCode::Palestine),
            Self::RepubliekNoordMacedonie => Some(IsoCountryCode::NorthMacedonia),
            Self::Moldavie => Some(IsoCountryCode::Moldova),
            Self::Burundi => Some(IsoCountryCode::Burundi),
            Self::Finland => Some(IsoCountryCode::Finland),
            Self::Griekenland => Some(IsoCountryCode::Greece),
            Self::Guatemala => Some(IsoCountryCode::Guatemala),
            Self::Nigeria => Some(IsoCountryCode::Nigeria),
            Self::Libie => Some(IsoCountryCode::Libya),
            Self::Ierland => Some(IsoCountryCode::Ireland),
            Self::Brazilie => Some(IsoCountryCode::Brazil),
            Self::Rwanda => Some(IsoCountryCode::Rwanda),
            Self::Venezuela => Some(IsoCountryCode::Venezuela),
            Self::IJsland => Some(IsoCountryCode::Iceland),
            Self::Liechtenstein => Some(IsoCountryCode::Liechtenstein),
            Self::Somalie => Some(IsoCountryCode::Somalia),
            Self::VerenigdeStatenVanAmerika => Some(IsoCountryCode::UnitedStates),
            Self::Bolivia => Some(IsoCountryCode::Bolivia),
            Self::Australie => Some(IsoCountryCode::Australia),
            Self::Jamaica => Some(IsoCountryCode::Jamaica),
            Self::Luxemburg => Some(IsoCountryCode::Luxembourg),
            Self::Tsjaad => Some(IsoCountryCode::Chad),
            Self::Mauritanie => Some(IsoCountryCode::Mauritania),
            Self::Kirgizie => Some(IsoCountryCode::Kyrgyzstan),
            Self::China => Some(IsoCountryCode::China),
            Self::Afghanistan => Some(IsoCountryCode::Afghanistan),
            Self::Indonesie => Some(IsoCountryCode::Indonesia),
            Self::Guyana => Some(IsoCountryCode::Guyana),
            // North Vietnam merged into Vietnam on 02-07-1976.
            Self::NoordVietnam => Some(IsoCountryCode::Vietnam),
            Self::Noorwegen => Some(IsoCountryCode::Norway),
            Self::SanMarino => Some(IsoCountryCode::SanMarino),
            // Historical code, superseded after reunification by code "9089".
            Self::Duitsland => Some(IsoCountryCode::Germany),
            Self::Nederland => Some(IsoCountryCode::Netherlands),
            Self::Cambodja => Some(IsoCountryCode::Cambodia),
            Self::Fiji => Some(IsoCountryCode::Fiji),
            Self::Bahamas => Some(IsoCountryCode::Bahamas),
            Self::Israel => Some(IsoCountryCode::Israel),
            Self::Nepal => Some(IsoCountryCode::Nepal),
            Self::ZuidKorea => Some(IsoCountryCode::SouthKorea),
            Self::Spanje => Some(IsoCountryCode::Spain),
            Self::Oekraine => Some(IsoCountryCode::Ukraine),
            Self::VerenigdKoninkrijk => Some(IsoCountryCode::UnitedKingdom),
            Self::Niger => Some(IsoCountryCode::Niger),
            Self::Haiti => Some(IsoCountryCode::Haiti),
            Self::Jordanie => Some(IsoCountryCode::Jordan),
            Self::Turkije => Some(IsoCountryCode::Turkiye),
            Self::TrinidadEnTobago => Some(IsoCountryCode::TrinidadAndTobago),
            // Yugoslavia split into multiple current countries; no single alpha-2 code is correct (RvIG lists the
            // withdrawn ISO code "YU"; not adopted here).
            Self::Joegoslavie => None,
            // Upper Volta was renamed Burkina Faso on 04-08-1984.
            Self::OpperVolta => Some(IsoCountryCode::BurkinaFaso),
            Self::Algerije => Some(IsoCountryCode::Algeria),
            Self::Gabon => Some(IsoCountryCode::Gabon),
            Self::NoordKorea => Some(IsoCountryCode::NorthKorea),
            Self::Oezbekistan => Some(IsoCountryCode::Uzbekistan),
            Self::SierraLeone => Some(IsoCountryCode::SierraLeone),
            // British Honduras was renamed Belize.
            Self::BritsHonduras => Some(IsoCountryCode::Belize),
            // The Canary Islands are an autonomous community of Spain, not separately coded in ISO 3166-1.
            Self::CanarischeEilanden => Some(IsoCountryCode::Spain),
            Self::FransPolynesie => Some(IsoCountryCode::FrenchPolynesia),
            Self::Gibraltar => Some(IsoCountryCode::Gibraltar),
            // Portuguese Timor is the predecessor territory of Timor-Leste.
            Self::PortugeesTimor => Some(IsoCountryCode::TimorLeste),
            Self::Tadzjikistan => Some(IsoCountryCode::Tajikistan),
            // The British Solomon Islands became the Solomon Islands.
            Self::BritseSalomonseilanden => Some(IsoCountryCode::SolomonIslands),
            Self::SaoTomeEnPrincipe => Some(IsoCountryCode::SaoTomeAndPrincipe),
            // Part of the single territory Saint Helena, Ascension and Tristan da Cunha.
            Self::SintHelena => Some(IsoCountryCode::SaintHelenaAscensionAndTristanDaCunha),
            // Part of the single territory Saint Helena, Ascension and Tristan da Cunha.
            Self::TristanDaCunha => Some(IsoCountryCode::SaintHelenaAscensionAndTristanDaCunha),
            // Western Samoa was renamed Samoa on 04-07-1997.
            Self::WestSamoa => Some(IsoCountryCode::Samoa),
            Self::Turkmenistan => Some(IsoCountryCode::Turkmenistan),
            Self::Georgie => Some(IsoCountryCode::Georgia),
            Self::BosnieEnHerzegovina => Some(IsoCountryCode::BosniaAndHerzegovina),
            Self::Tsjechie => Some(IsoCountryCode::Czechia),
            Self::Slowakije => Some(IsoCountryCode::Slovakia),
            // The Federal Republic of Yugoslavia (Serbia and Montenegro) split into Serbia and Montenegro; no single
            // successor (RvIG lists the withdrawn ISO code "YU"; not adopted here).
            Self::FederaleRepubliekJoegoslavie => None,
            Self::DemocratischeRepubliekCongo => Some(IsoCountryCode::DemocraticRepublicOfTheCongo),
            Self::Oeganda => Some(IsoCountryCode::Uganda),
            Self::Kenia => Some(IsoCountryCode::Kenya),
            Self::Malta => Some(IsoCountryCode::Malta),
            Self::Barbados => Some(IsoCountryCode::Barbados),
            Self::Andorra => Some(IsoCountryCode::Andorra),
            Self::Mexico => Some(IsoCountryCode::Mexico),
            Self::CostaRica => Some(IsoCountryCode::CostaRica),
            Self::Gambia => Some(IsoCountryCode::Gambia),
            Self::Syrie => Some(IsoCountryCode::Syria),
            // The Netherlands Antilles dissolved in 2010 into Curaçao, Sint Maarten and the "BQ" special
            // municipalities; no single successor.
            Self::NederlandseAntillen => None,
            // South Yemen merged into Yemen on 22-05-1990.
            Self::ZuidJemen => Some(IsoCountryCode::Yemen),
            Self::Egypte => Some(IsoCountryCode::Egypt),
            Self::Argentinie => Some(IsoCountryCode::Argentina),
            Self::Lesotho => Some(IsoCountryCode::Lesotho),
            Self::Honduras => Some(IsoCountryCode::Honduras),
            Self::Nicaragua => Some(IsoCountryCode::Nicaragua),
            Self::Pakistan => Some(IsoCountryCode::Pakistan),
            Self::Senegal => Some(IsoCountryCode::Senegal),
            // Dahomey was renamed Benin.
            Self::Dahomey => Some(IsoCountryCode::Benin),
            Self::Bulgarije => Some(IsoCountryCode::Bulgaria),
            Self::Maleisie => Some(IsoCountryCode::Malaysia),
            Self::DominicaanseRepubliek => Some(IsoCountryCode::DominicanRepublic),
            Self::Polen => Some(IsoCountryCode::Poland),
            // Pre-revolutionary Imperial Russia; substantially succeeded by the Soviet Union rather than directly by
            // the modern Russian Federation, so no single current code is adopted.
            Self::RuslandOud => None,
            Self::BritseMaagdeneilanden => Some(IsoCountryCode::BritishVirginIslands),
            Self::Tanzania => Some(IsoCountryCode::Tanzania),
            Self::ElSalvador => Some(IsoCountryCode::ElSalvador),
            Self::SriLanka => Some(IsoCountryCode::SriLanka),
            Self::Soedan => Some(IsoCountryCode::Sudan),
            Self::Japan => Some(IsoCountryCode::Japan),
            // Hong Kong retains its own ISO 3166-1 code as a special administrative region.
            Self::Hongkong => Some(IsoCountryCode::HongKong),
            Self::Panama => Some(IsoCountryCode::Panama),
            Self::Uruguay => Some(IsoCountryCode::Uruguay),
            Self::Ecuador => Some(IsoCountryCode::Ecuador),
            Self::Guinee => Some(IsoCountryCode::Guinea),
            Self::Maldiven => Some(IsoCountryCode::Maldives),
            Self::Thailand => Some(IsoCountryCode::Thailand),
            Self::Libanon => Some(IsoCountryCode::Lebanon),
            Self::Italie => Some(IsoCountryCode::Italy),
            Self::Koeweit => Some(IsoCountryCode::Kuwait),
            Self::India => Some(IsoCountryCode::India),
            Self::Roemenie => Some(IsoCountryCode::Romania),
            // Czechoslovakia split into Czechia and Slovakia; no single alpha-2 code is correct.
            Self::TsjechoSlowakije => None,
            Self::Peru => Some(IsoCountryCode::Peru),
            Self::Portugal => Some(IsoCountryCode::Portugal),
            Self::Oman => Some(IsoCountryCode::Oman),
            Self::Mongolie => Some(IsoCountryCode::Mongolia),
            Self::Samoa => Some(IsoCountryCode::Samoa),
            Self::VerenigdeArabischeEmiraten => Some(IsoCountryCode::UnitedArabEmirates),
            // Tibet has no official ISO 3166-1 code; deferred as a politically sensitive case rather than unilaterally
            // mapping it to China.
            Self::Tibet => None,
            Self::Nauru => Some(IsoCountryCode::Nauru),
            // Dutch New Guinea was transferred to Indonesia in 1963.
            Self::NederlandsNieuwGuinea => Some(IsoCountryCode::Indonesia),
            // Tanganyika merged with Zanzibar to form Tanzania in 1964.
            Self::Tanganyika => Some(IsoCountryCode::Tanzania),
            Self::Palestina => Some(IsoCountryCode::Palestine),
            // The British West Indies devolved into multiple current Caribbean countries; no single successor.
            Self::BritsWestIndie => None,
            // Portuguese Africa devolved into multiple current countries (Angola, Mozambique, Guinea-Bissau, etc.); no
            // single successor.
            Self::PortugeesAfrika => None,
            Self::Letland => Some(IsoCountryCode::Latvia),
            Self::Estland => Some(IsoCountryCode::Estonia),
            Self::Litouwen => Some(IsoCountryCode::Lithuania),
            // British Africa devolved into multiple current countries; no single successor.
            Self::BritsAfrika => None,
            // The Belgian Congo became Congo, now the Democratic Republic of the Congo.
            Self::BelgischCongo => Some(IsoCountryCode::DemocraticRepublicOfTheCongo),
            // British India was partitioned into India, Pakistan and (later) Bangladesh; no single successor.
            Self::BritsIndie => None,
            // Northern Rhodesia became Zambia.
            Self::NoordRhodesie => Some(IsoCountryCode::Zambia),
            // Southern Rhodesia became Rhodesia, later Zimbabwe.
            Self::ZuidRhodesie => Some(IsoCountryCode::Zimbabwe),
            // The Saar Protectorate rejoined (West) Germany in 1957.
            Self::Saarland => Some(IsoCountryCode::Germany),
            // French Indochina split into Vietnam, Laos and Cambodia; no single successor.
            Self::FransIndochina => None,
            // British West Borneo became part of Malaysia (Sarawak).
            Self::BritsWestBorneo => Some(IsoCountryCode::Malaysia),
            // The Gold Coast was renamed Ghana upon independence.
            Self::Goudkust => Some(IsoCountryCode::Ghana),
            // Ras al-Khaimah is one of the emirates of the United Arab Emirates.
            Self::RasAlKhaimah => Some(IsoCountryCode::UnitedArabEmirates),
            // French Congo became the Republic of the Congo.
            Self::FransCongo => Some(IsoCountryCode::Congo),
            // Siam was renamed Thailand.
            Self::Siam => Some(IsoCountryCode::Thailand),
            // British East Africa devolved into multiple current countries (Kenya, Uganda, Tanzania); no single
            // successor.
            Self::BritsOostAfrika => None,
            // British North Borneo became Sabah, part of Malaysia.
            Self::BritsNoordBorneo => Some(IsoCountryCode::Malaysia),
            Self::Bangladesh => Some(IsoCountryCode::Bangladesh),
            // East Germany merged into reunified Germany on 03-10-1990.
            Self::DuitseDemocratischeRepubliek => Some(IsoCountryCode::Germany),
            // Madeira is an autonomous region of Portugal, not separately coded in ISO 3166-1.
            Self::Madeira => Some(IsoCountryCode::Portugal),
            Self::AmerikaanseMaagdeneilanden => Some(IsoCountryCode::UsVirginIslands),
            // The Australian-administered Solomon Islands territory is now part of Solomon Islands.
            Self::AustralischeSalomonseilanden => Some(IsoCountryCode::SolomonIslands),
            // Spanish Sahara became Western Sahara.
            Self::SpaanseSahara => Some(IsoCountryCode::WesternSahara),
            Self::Kaaimaneilanden => Some(IsoCountryCode::CaymanIslands),
            // Part of the single territory Turks and Caicos Islands.
            Self::Caicoseilanden => Some(IsoCountryCode::TurksAndCaicosIslands),
            // Part of the single territory Turks and Caicos Islands.
            Self::Turkseilanden => Some(IsoCountryCode::TurksAndCaicosIslands),
            // The British Antarctic Territory is a claim within Antarctica.
            Self::BritsAntarctischTerritorium => Some(IsoCountryCode::Antarctica),
            Self::BritsIndischeOceaanterritorium => Some(IsoCountryCode::BritishIndianOceanTerritory),
            Self::Cookeilanden => Some(IsoCountryCode::CookIslands),
            Self::Tokelau => Some(IsoCountryCode::Tokelau),
            Self::NieuwCaledonie => Some(IsoCountryCode::NewCaledonia),
            // Hawaii became a US state in 1959.
            Self::HawaiiEilanden => Some(IsoCountryCode::UnitedStates),
            Self::Guam => Some(IsoCountryCode::Guam),
            Self::AmerikaansSamoa => Some(IsoCountryCode::AmericanSamoa),
            // Midway Atoll is one of the United States Minor Outlying Islands.
            Self::Midway => Some(IsoCountryCode::UnitedStatesMinorOutlyingIslands),
            // The Ryukyu Islands (Okinawa) reverted to Japan in 1972.
            Self::RiukiuEilanden => Some(IsoCountryCode::Japan),
            // Wake Island is one of the United States Minor Outlying Islands.
            Self::Wake => Some(IsoCountryCode::UnitedStatesMinorOutlyingIslands),
            // The Pacific Islands Trust Territory devolved into multiple current countries (Palau, Micronesia, Marshall
            // Islands, Northern Mariana Islands); no single successor.
            Self::PacificEilanden => None,
            Self::Grenada => Some(IsoCountryCode::Grenada),
            Self::NoordelijkeMarianen => Some(IsoCountryCode::NorthernMarianaIslands),
            // Cabinda is an exclave province of Angola.
            Self::Cabinda => Some(IsoCountryCode::Angola),
            // Canton and Enderbury Islands are now part of Kiribati.
            Self::CantonEnEnderbury => Some(IsoCountryCode::Kiribati),
            Self::Christmaseiland => Some(IsoCountryCode::ChristmasIsland),
            Self::Cocoseilanden => Some(IsoCountryCode::CocosKeelingIslands),
            Self::Faeroer => Some(IsoCountryCode::FaroeIslands),
            Self::Montserrat => Some(IsoCountryCode::Montserrat),
            Self::Norfolk => Some(IsoCountryCode::NorfolkIsland),
            Self::Belize => Some(IsoCountryCode::Belize),
            // Tasmania is a state of Australia, not separately coded in ISO 3166-1.
            Self::Tasmanie => Some(IsoCountryCode::Australia),
            Self::TurksEnCaicoseilanden => Some(IsoCountryCode::TurksAndCaicosIslands),
            Self::PuertoRico => Some(IsoCountryCode::PuertoRico),
            Self::PapoeaNieuwGuinea => Some(IsoCountryCode::PapuaNewGuinea),
            Self::Salomonseilanden => Some(IsoCountryCode::SolomonIslands),
            Self::Benin => Some(IsoCountryCode::Benin),
            Self::Vietnam => Some(IsoCountryCode::Vietnam),
            Self::Kaapverdie => Some(IsoCountryCode::CaboVerde),
            Self::Seychellen => Some(IsoCountryCode::Seychelles),
            Self::Kiribati => Some(IsoCountryCode::Kiribati),
            Self::Tuvalu => Some(IsoCountryCode::Tuvalu),
            Self::SaintLucia => Some(IsoCountryCode::SaintLucia),
            Self::Dominica => Some(IsoCountryCode::Dominica),
            Self::Zimbabwe => Some(IsoCountryCode::Zimbabwe),
            // Dubai is one of the emirates of the United Arab Emirates.
            Self::Dubai => Some(IsoCountryCode::UnitedArabEmirates),
            // The New Hebrides became Vanuatu upon independence.
            Self::NieuweHebriden => Some(IsoCountryCode::Vanuatu),
            // The Channel Islands comprise two separate current jurisdictions (Jersey and Guernsey); no single
            // successor.
            Self::Kanaaleilanden => None,
            Self::Man => Some(IsoCountryCode::IsleOfMan),
            Self::Anguilla => Some(IsoCountryCode::Anguilla),
            Self::SaintKittsEnNevis => Some(IsoCountryCode::SaintKittsAndNevis),
            // Antigua is now part of Antigua and Barbuda.
            Self::Antigua => Some(IsoCountryCode::AntiguaAndBarbuda),
            // Saint Vincent is now part of Saint Vincent and the Grenadines.
            Self::SaintVincent => Some(IsoCountryCode::SaintVincentAndTheGrenadines),
            // The Gilbert Islands became Kiribati.
            Self::Gilberteilanden => Some(IsoCountryCode::Kiribati),
            // The Panama Canal Zone reverted to Panama.
            Self::Panamakanaalzone => Some(IsoCountryCode::Panama),
            // Anguilla later separated; the remainder became Saint Kitts and Nevis.
            Self::SaintKittsNevisEnAnguilla => Some(IsoCountryCode::SaintKittsAndNevis),
            // Belau is the former name for Palau.
            Self::Belau => Some(IsoCountryCode::Palau),
            Self::Palau => Some(IsoCountryCode::Palau),
            Self::AntiguaEnBarbuda => Some(IsoCountryCode::AntiguaAndBarbuda),
            Self::SintHelenaAscensionEnTristanDaCunha => Some(IsoCountryCode::SaintHelenaAscensionAndTristanDaCunha),
            Self::AmerikaanseKleinereAfgelegenEilanden => Some(IsoCountryCode::UnitedStatesMinorOutlyingIslands),
            Self::ZuidGeorgiaEnDeZuidelijkeSandwicheilanden => {
                Some(IsoCountryCode::SouthGeorgiaAndTheSouthSandwichIslands)
            }
            Self::SaintBarthelemy => Some(IsoCountryCode::SaintBarthelemy),
            Self::SaintMartin => Some(IsoCountryCode::SaintMartin),
            Self::Guernsey => Some(IsoCountryCode::Guernsey),
            Self::Jersey => Some(IsoCountryCode::Jersey),
            Self::Aland => Some(IsoCountryCode::AlandIslands),
            // Newfoundland joined Canada as a province in 1949.
            Self::Newfoundland => Some(IsoCountryCode::Canada),
            // Nyasaland became Malawi.
            Self::Nyasaland => Some(IsoCountryCode::Malawi),
            Self::Eritrea => Some(IsoCountryCode::Eritrea),
            // Ifni was ceded to Morocco in 1969.
            Self::Ifni => Some(IsoCountryCode::Morocco),
            // British Cameroons split between Nigeria and Cameroon; no single successor.
            Self::BritsKameroen => None,
            // Kaiser-Wilhelmsland (German New Guinea) is now part of Papua New Guinea.
            Self::KeizerWilhelmsland => Some(IsoCountryCode::PapuaNewGuinea),
            Self::RepubliekCongo => Some(IsoCountryCode::Congo),
            // Early name for the present-day Democratic Republic of the Congo.
            Self::CongoKinshasa => Some(IsoCountryCode::DemocraticRepublicOfTheCongo),
            Self::Madagaskar => Some(IsoCountryCode::Madagascar),
            Self::CongoBrazzaville => Some(IsoCountryCode::Congo),
            // The Leeward Islands comprise multiple current Caribbean jurisdictions; no single successor.
            Self::Leewardeilanden => None,
            // The Windward Islands comprise multiple current Caribbean jurisdictions; no single successor.
            Self::Windwardeilanden => None,
            // The French Territory of the Afars and the Issas became Djibouti.
            Self::FransTerritoriumVoorAfarsEnIssas => Some(IsoCountryCode::Djibouti),
            // The Phoenix Islands are now part of Kiribati.
            Self::Phoenixeilanden => Some(IsoCountryCode::Kiribati),
            // Portuguese Guinea became Guinea-Bissau.
            Self::PortugeesGuinee => Some(IsoCountryCode::GuineaBissau),
            // German South West Africa, early name for present-day Namibia.
            Self::DuitsZuidwestAfrika => Some(IsoCountryCode::Namibia),
            Self::Namibie => Some(IsoCountryCode::Namibia),
            // British Somaliland merged into Somalia.
            Self::BritsSomaliland => Some(IsoCountryCode::Somalia),
            // Italian Somaliland merged into Somalia.
            Self::ItaliaansSomaliland => Some(IsoCountryCode::Somalia),
            // The Dutch East Indies became Indonesia.
            Self::NederlandsIndie => Some(IsoCountryCode::Indonesia),
            // British Guiana became Guyana.
            Self::BritsGuyana => Some(IsoCountryCode::Guyana),
            // Swaziland was renamed Eswatini on 19-04-2018.
            Self::Swaziland => Some(IsoCountryCode::Eswatini),
            Self::Qatar => Some(IsoCountryCode::Qatar),
            Self::Eswatini => Some(IsoCountryCode::Eswatini),
            // Aden, part of South Yemen, is now part of unified Yemen.
            Self::Aden => Some(IsoCountryCode::Yemen),
            // The Federation of South Arabia, predecessor of South Yemen, is now part of unified Yemen.
            Self::ZuidArabischeFederatie => Some(IsoCountryCode::Yemen),
            Self::EquatoriaalGuinea => Some(IsoCountryCode::EquatorialGuinea),
            // Spanish Guinea became Equatorial Guinea.
            Self::SpaansGuinee => Some(IsoCountryCode::EquatorialGuinea),
            // The United Arab Republic (Egypt-Syria union) dissolved into separate Egypt and Syria; no single
            // successor.
            Self::VerenigdeArabischeRepubliek => None,
            Self::Bermuda => Some(IsoCountryCode::Bermuda),
            // The Soviet Union split into multiple current countries; no single alpha-2 code is correct.
            Self::SovjetUnie => None,
            // German East Africa devolved into multiple current countries (Tanzania, Rwanda, Burundi); no single
            // successor.
            Self::DuitsOostAfrika => None,
            // Zanzibar merged with Tanganyika to form Tanzania.
            Self::Zanzibar => Some(IsoCountryCode::Tanzania),
            // Ceylon was renamed Sri Lanka.
            Self::Ceylon => Some(IsoCountryCode::SriLanka),
            // Muscat and Oman was renamed Oman.
            Self::MuscatEnOman => Some(IsoCountryCode::Oman),
            // Trucial Oman (the Trucial States) became the United Arab Emirates.
            Self::TrucialOman => Some(IsoCountryCode::UnitedArabEmirates),
            // Indochina split into Vietnam, Laos and Cambodia; no single successor.
            Self::Indochina => None,
            Self::Marshalleilanden => Some(IsoCountryCode::MarshallIslands),
            // Sarawak is part of Malaysia.
            Self::Sarawak => Some(IsoCountryCode::Malaysia),
            // British Borneo territories are now part of Malaysia.
            Self::BritsBorneo => Some(IsoCountryCode::Malaysia),
            // Sabah is part of Malaysia.
            Self::Sabah => Some(IsoCountryCode::Malaysia),
            // Abu Dhabi is one of the emirates of the United Arab Emirates.
            Self::AbuDhabi => Some(IsoCountryCode::UnitedArabEmirates),
            // Ajman is one of the emirates of the United Arab Emirates.
            Self::Ajman => Some(IsoCountryCode::UnitedArabEmirates),
            // Basutoland became Lesotho.
            Self::Basutoland => Some(IsoCountryCode::Lesotho),
            // Bechuanaland became Botswana.
            Self::Bechuanaland => Some(IsoCountryCode::Botswana),
            // Fujairah is one of the emirates of the United Arab Emirates.
            Self::Fujairah => Some(IsoCountryCode::UnitedArabEmirates),
            // French Cameroons became Cameroon.
            Self::FransKameroen => Some(IsoCountryCode::Cameroon),
            // Pre-federation Malay state, now part of Malaysia.
            Self::Johore => Some(IsoCountryCode::Malaysia),
            // Korea split into North and South Korea; no single alpha-2 code is correct.
            Self::Korea => None,
            // Labuan is part of Malaysia.
            Self::Labuan => Some(IsoCountryCode::Malaysia),
            // Umm al-Qaiwain is one of the emirates of the United Arab Emirates.
            Self::UmmAlQaiwain => Some(IsoCountryCode::UnitedArabEmirates),
            // Austria-Hungary split into multiple current countries; no single successor.
            Self::OostenrijkHongarije => None,
            // Portuguese East Africa became Mozambique.
            Self::PortugeesOostAfrika => Some(IsoCountryCode::Mozambique),
            // Portuguese West Africa became Angola.
            Self::PortugeesWestAfrika => Some(IsoCountryCode::Angola),
            // Sharjah is one of the emirates of the United Arab Emirates.
            Self::Sharjah => Some(IsoCountryCode::UnitedArabEmirates),
            // The Straits Settlements split between Malaysia (Penang, Malacca) and Singapore; no single successor.
            Self::StraitsSettlements => None,
            // Abyssinia is the historical name for Ethiopia.
            Self::Abessinie => Some(IsoCountryCode::Ethiopia),
            // French West Africa devolved into multiple current West African countries; no single successor.
            Self::FransWestAfrika => None,
            // French Equatorial Africa devolved into multiple current Central African countries; no single successor.
            Self::FransEquatoriaalAfrika => None,
            // Urundi, part of Ruanda-Urundi, is now Burundi.
            Self::Urundi => Some(IsoCountryCode::Burundi),
            // Ruanda-Urundi split into Rwanda and Burundi; no single successor.
            Self::RuandaUrundi => None,
            // Goa was annexed by India in 1961.
            Self::Goa => Some(IsoCountryCode::India),
            // The Free City of Danzig is now Gdańsk, Poland.
            Self::Dantzig => Some(IsoCountryCode::Poland),
            Self::CentraalAfrikaanseRepubliek => Some(IsoCountryCode::CentralAfricanRepublic),
            Self::Djibouti => Some(IsoCountryCode::Djibouti),
            // Transjordan was renamed Jordan.
            Self::Transjordanie => Some(IsoCountryCode::Jordan),
            Self::BondsrepubliekDuitsland => Some(IsoCountryCode::Germany),
            Self::Vanuatu => Some(IsoCountryCode::Vanuatu),
            Self::Niue => Some(IsoCountryCode::Niue),
            // Spanish North Africa (Ceuta, Melilla) are autonomous cities of Spain, not separately coded in ISO 3166-1.
            Self::SpaansNoordAfrika => Some(IsoCountryCode::Spain),
            Self::WestelijkeSahara => Some(IsoCountryCode::WesternSahara),
            Self::Micronesie => Some(IsoCountryCode::Micronesia),
            Self::Svalbardeilanden => Some(IsoCountryCode::SvalbardAndJanMayen),
            // International territory/high seas is not a country.
            Self::InternationaalGebied => None,
            Self::Overig => None,
        }
    }

    /// The Rulebook `country` value for this code (`resident_country`/`place_of_birth.country`). Kosovo has no
    /// official ISO 3166-1 code, but "XK" is widely used in practice (see [`RulebookCountryCode::Other`]). Returns
    /// `None` for a country that can't be mapped to either.
    fn to_rulebook_country(&self) -> Option<RulebookCountryCode> {
        match self {
            Self::Kosovo => Some(RulebookCountryCode::Other("XK".to_owned())),
            _ => self.alpha2().map(RulebookCountryCode::Iso),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct BrpResidence {
    #[serde(rename = "verblijfadres")]
    address: BrpAddress,
}

#[derive(Clone, Deserialize)]
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

    #[serde(rename = "postcode")]
    postal_code: String,

    #[serde(rename = "woonplaats")]
    city: String,

    #[serde(rename = "land", default = "default_country")]
    country: BrpCountryDetail,
}

fn default_country() -> BrpCountryDetail {
    BrpCountryDetail {
        code: RvigCountryCode::Nederland,
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

        format!("{}{}{}", self.house_number, house_letter, house_number_addition)
    }

    /// `resident_street` includes the house number in its value
    fn mdoc_street(&self) -> Option<String> {
        self.street()
            .map(|street| format!("{street} {}", self.locator_designator()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use attestation_data::attributes::Attribute;
    use rstest::rstest;
    use serde_json::json;
    use utils::path::prefix_local_path;

    use crate::pid::brp::data::BrpPersons;
    use crate::pid::rulebook::country_code::IsoCountryCode;
    use crate::pid::rulebook::issuer_metadata::PidIssuerMetadata;

    fn read_json(name: &str) -> String {
        fs::read_to_string(prefix_local_path(PathBuf::from(format!(
            "resources/test/haal-centraal-examples/{name}.json"
        ))))
        .unwrap()
    }

    #[test]
    fn should_deserialize_empty_response() {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json("empty")).unwrap();
        assert!(brp_persons.persons.is_empty());
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
    fn should_handle_house_number(#[case] json_file_name: &str, #[case] expected_house_number: &str) {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json(json_file_name)).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert_eq!(expected_house_number, brp_person.residence.address.locator_designator());
    }

    #[rstest]
    #[case("naamgebruik-eigen", "van Buren")]
    #[case("naamgebruik-partner", "Postma")]
    #[case("naamgebruik-partner-first", "ten Hag-van Buren")]
    #[case("naamgebruik-partner-last", "van Buren-ten Hag")]
    #[case("prefix-family-name", "van Buren")]
    // Naamgebruik indicates the partner's name should be used, but there is no current partner on record: falls
    // back to the own name rather than erroring.
    #[case("naamgebruik-illegal", "van Buren")]
    fn should_handle_naamgebruik(#[case] json_file_name: &str, #[case] expected_family_name: &str) {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json(json_file_name)).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert_eq!(expected_family_name, brp_person.family_name());
    }

    #[test]
    fn should_convert_brp_person_to_sd_jwt_issuable_vec() {
        let mut brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let issuer_metadata = PidIssuerMetadata {
            issuing_country: IsoCountryCode::Netherlands,
            issuing_authority: "Test Issuing Authority".to_owned(),
        };
        let pid_card = brp_persons
            .persons
            .remove(0)
            .into_sd_jwt_attributes(Attribute::Text(String::from("recovery_code_123")), issuer_metadata);

        assert_eq!(
            json!({
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
                "bsn": {
                    "type": "text",
                    "value": "999991772"
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
            serde_json::to_value(pid_card).unwrap()
        );
    }

    #[test]
    fn should_convert_brp_person_to_mdoc_issuable_vec() {
        let mut brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let issuer_metadata = PidIssuerMetadata {
            issuing_country: IsoCountryCode::Netherlands,
            issuing_authority: "Test Issuing Authority".to_owned(),
        };
        let pid_card = brp_persons
            .persons
            .remove(0)
            .into_mdoc_attributes(Attribute::Text("recovery_code_123".to_owned()), issuer_metadata);

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
                        "bsn": {
                            "type": "text",
                            "value": "999991772"
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
                        "recovery_code": {
                            "type": "text",
                            "value": "recovery_code_123"
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
                }
            }),
            serde_json::to_value(pid_card).unwrap()
        );
    }
}
