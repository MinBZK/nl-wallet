use attestation_data::attributes::Attribute;
use attestation_data::attributes::Attributes;
use chrono::NaiveDate;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde_with::skip_serializing_none;

use crate::pid::rulebook::constants::PID_ADDRESS_GROUP;
use crate::pid::rulebook::constants::PID_AGE_OVER_18;
use crate::pid::rulebook::constants::PID_BIRTH_DATE;
use crate::pid::rulebook::constants::PID_BSN;
use crate::pid::rulebook::constants::PID_FAMILY_NAME;
use crate::pid::rulebook::constants::PID_GIVEN_NAME;
use crate::pid::rulebook::constants::PID_MDOC_NAMESPACE;
use crate::pid::rulebook::constants::PID_NATIONALITY;
use crate::pid::rulebook::constants::PID_RECOVERY_CODE;
use crate::pid::rulebook::constants::PID_RESIDENT_CITY;
use crate::pid::rulebook::constants::PID_RESIDENT_COUNTRY;
use crate::pid::rulebook::constants::PID_RESIDENT_HOUSE_NUMBER;
use crate::pid::rulebook::constants::PID_RESIDENT_POSTAL_CODE;
use crate::pid::rulebook::constants::PID_RESIDENT_STREET;
use crate::pid::rulebook::constants::PID_SEX;
use crate::pid::rulebook::country_code::CountryCode;
use crate::pid::rulebook::issuer_metadata::PidIssuerMetadata;
use crate::pid::rulebook::sex::PidSex;

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

    pub fn bsn(&self) -> &str {
        &self.bsn
    }
}

impl BrpPerson {
    pub fn into_sd_jwt_attributes(self, recovery_code: Attribute, issuer_metadata: PidIssuerMetadata) -> Attributes {
        let address_attributes = self.residence.clone().into_address_attributes();
        let mut attributes = self.common_attributes();

        attributes.extend(issuer_metadata.into_attributes());
        attributes.insert(String::from(PID_RECOVERY_CODE), recovery_code);
        attributes.insert(
            String::from(PID_ADDRESS_GROUP),
            Attribute::Object(IndexMap::from_iter(address_attributes)),
        );

        Attributes::from(attributes)
    }

    pub fn into_mdoc_attributes(self, recovery_code: Attribute, issuer_metadata: PidIssuerMetadata) -> Attributes {
        let address_attributes = self.residence.clone().into_address_attributes();
        let mut attributes = self.common_attributes();

        attributes.extend(issuer_metadata.into_attributes());
        attributes.insert(String::from(PID_RECOVERY_CODE), recovery_code);
        attributes.extend(address_attributes);

        Attributes::from(IndexMap::from([(
            PID_MDOC_NAMESPACE.to_owned(),
            Attribute::Object(attributes),
        )]))
    }

    fn common_attributes(self) -> IndexMap<String, Attribute> {
        let given_names = self.name.given_names.clone();
        let is_over_18 = self.is_over_18();
        let family_name = self.name.into_name_with_prefix();
        let sex = self.gender.map(|gender| Attribute::from(PidSex::from(gender.code)));
        let nationalities = self
            .nationalities
            .into_iter()
            .filter_map(|nationality| nationality.nationality)
            .map(|detail| detail.code.as_attribute())
            .collect_vec();

        IndexMap::from_iter(
            vec![
                Some((String::from(PID_FAMILY_NAME), Attribute::Text(family_name))),
                given_names.map(|names| (String::from(PID_GIVEN_NAME), Attribute::Text(names))),
                Some((
                    String::from(PID_BIRTH_DATE),
                    Attribute::Text(self.birth.date.date.format("%Y-%m-%d").to_string()),
                )),
                Some((String::from(PID_AGE_OVER_18), Attribute::Bool(is_over_18))),
                Some((String::from(PID_BSN), Attribute::Text(self.bsn))),
                Some((
                    String::from(PID_NATIONALITY),
                    Attribute::Array(nationalities.into_iter().map(Attribute::Text).collect_vec()),
                )),
                sex.map(|sex| (PID_SEX.to_owned(), sex)),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<(String, Attribute)>>(),
        )
    }
}

impl BrpResidence {
    pub fn into_address_attributes(self) -> impl Iterator<Item = (String, Attribute)> {
        vec![
            self.address
                .street()
                .map(|street| (String::from(PID_RESIDENT_STREET), Attribute::Text(String::from(street)))),
            Some((
                String::from(PID_RESIDENT_HOUSE_NUMBER),
                Attribute::Text(self.address.locator_designator()),
            )),
            Some((
                String::from(PID_RESIDENT_POSTAL_CODE),
                Attribute::Text(self.address.postal_code),
            )),
            Some((String::from(PID_RESIDENT_CITY), Attribute::Text(self.address.city))),
            Some((
                String::from(PID_RESIDENT_COUNTRY),
                Attribute::Text(self.address.country.description),
            )),
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

/// BRP's `geslacht.code` only ever yields "onbekend", "man" or "vrouw", so only [`PidSex::NotKnown`],
/// [`PidSex::Male`] and [`PidSex::Female`] are actually produced by this issuer, even though the Rulebook's `sex`
/// attribute defines a larger value set (see [`PidSex`]).
impl From<BrpGenderCode> for PidSex {
    fn from(value: BrpGenderCode) -> Self {
        match value {
            BrpGenderCode::O => PidSex::NotKnown,
            BrpGenderCode::M => PidSex::Male,
            BrpGenderCode::V => PidSex::Female,
        }
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

#[derive(Clone, Deserialize)]
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
    fn alpha2(&self) -> Option<CountryCode> {
        match self {
            Self::Onbekend => None,
            Self::Nederlandse => Some(CountryCode::Netherlands),
            Self::BehandeldAlsNederlander => None,
            Self::Slowaakse => Some(CountryCode::Slovakia),
            Self::Tsjechische => Some(CountryCode::Czechia),
            Self::BurgerVanBosnieHerzegovina => Some(CountryCode::BosniaAndHerzegovina),
            Self::Georgische => Some(CountryCode::Georgia),
            Self::Turkmeense => Some(CountryCode::Turkmenistan),
            Self::Tadzjiekse => Some(CountryCode::Tajikistan),
            Self::Oezbeekse => Some(CountryCode::Uzbekistan),
            Self::Oekrainense => Some(CountryCode::Ukraine),
            Self::Kirgizische => Some(CountryCode::Kyrgyzstan),
            Self::Moldavische => Some(CountryCode::Moldova),
            Self::Kazachse => Some(CountryCode::Kazakhstan),
            Self::Belarussische => Some(CountryCode::Belarus),
            Self::Azerbeidzjaanse => Some(CountryCode::Azerbaijan),
            Self::Armeense => Some(CountryCode::Armenia),
            Self::Russische => Some(CountryCode::Russia),
            Self::Sloveense => Some(CountryCode::Slovenia),
            Self::Kroatische => Some(CountryCode::Croatia),
            Self::Letse => Some(CountryCode::Latvia),
            Self::Estische => Some(CountryCode::Estonia),
            Self::Litouwse => Some(CountryCode::Lithuania),
            Self::Marshalleilandse => Some(CountryCode::MarshallIslands),
            Self::Myanmarese => Some(CountryCode::Myanmar),
            Self::Namibische => Some(CountryCode::Namibia),
            Self::Albanese => Some(CountryCode::Albania),
            Self::Andorrese => Some(CountryCode::Andorra),
            Self::Belgische => Some(CountryCode::Belgium),
            Self::Bulgaarse => Some(CountryCode::Bulgaria),
            Self::Deense => Some(CountryCode::Denmark),
            Self::BurgerVanDeBondsrepubliekDuitsland => Some(CountryCode::Germany),
            Self::Finse => Some(CountryCode::Finland),
            Self::Franse => Some(CountryCode::France),
            Self::Jemenitische => Some(CountryCode::Yemen),
            Self::Griekse => Some(CountryCode::Greece),
            Self::BritsBurger => Some(CountryCode::UnitedKingdom),
            Self::Hongaarse => Some(CountryCode::Hungary),
            Self::Ierse => Some(CountryCode::Ireland),
            Self::IJslandse => Some(CountryCode::Iceland),
            Self::Italiaanse => Some(CountryCode::Italy),
            // Yugoslavia (ended 01-02-2004) split into multiple current countries
            Self::Joegoslavische => None,
            Self::Liechtensteinse => Some(CountryCode::Liechtenstein),
            Self::Luxemburgse => Some(CountryCode::Luxembourg),
            Self::Maltese => Some(CountryCode::Malta),
            Self::Monegaskische => Some(CountryCode::Monaco),
            Self::Noorse => Some(CountryCode::Norway),
            Self::Oostenrijkse => Some(CountryCode::Austria),
            Self::Poolse => Some(CountryCode::Poland),
            Self::Portugese => Some(CountryCode::Portugal),
            Self::Roemeense => Some(CountryCode::Romania),
            // The Soviet Union split into multiple current countries (ended 31-12-1991).
            Self::BurgerVanDeSovjetUnie => None,
            Self::SanMarinese => Some(CountryCode::SanMarino),
            Self::Spaanse => Some(CountryCode::Spain),
            // Czechoslovakia split into Czechia and Slovakia on 01-01-1993.
            Self::TsjechoSlowaakse => None,
            Self::Vaticaanse => Some(CountryCode::HolySee),
            Self::Zweedse => Some(CountryCode::Sweden),
            Self::Zwitserse => Some(CountryCode::Switzerland),
            // Eastern Germany merged into reunified Germany.
            Self::OostDuitse => Some(CountryCode::Germany),
            // A British-nationality-law status.
            Self::BritsOnderdaan => Some(CountryCode::UnitedKingdom),
            Self::Eritrese => Some(CountryCode::Eritrea),
            // A British-nationality-law status.
            Self::BritsOverzeesBurger => Some(CountryCode::UnitedKingdom),
            Self::Macedonische => Some(CountryCode::NorthMacedonia),
            // Kosovo has no official ISO 3166-1 code, see [`Self::as_nationality_attribute`].
            Self::Kosovaarse => None,
            Self::BurgerVanNoordMacedonie => Some(CountryCode::NorthMacedonia),
            Self::Algerijnse => Some(CountryCode::Algeria),
            Self::Angolese => Some(CountryCode::Angola),
            Self::Burundese => Some(CountryCode::Burundi),
            Self::Botswaanse => Some(CountryCode::Botswana),
            Self::Burkinese => Some(CountryCode::BurkinaFaso),
            Self::CentraalAfrikaanse => Some(CountryCode::CentralAfricanRepublic),
            Self::Comorese => Some(CountryCode::Comoros),
            Self::BurgerVanCongo => Some(CountryCode::Congo),
            Self::Beninse => Some(CountryCode::Benin),
            Self::Egyptische => Some(CountryCode::Egypt),
            Self::EquatoriaalGuinese => Some(CountryCode::EquatorialGuinea),
            Self::Ethiopische => Some(CountryCode::Ethiopia),
            Self::Djiboutiaanse => Some(CountryCode::Djibouti),
            Self::Gabonese => Some(CountryCode::Gabon),
            Self::Gambiaanse => Some(CountryCode::Gambia),
            Self::Ghanese => Some(CountryCode::Ghana),
            Self::Guinese => Some(CountryCode::Guinea),
            Self::Ivoriaanse => Some(CountryCode::CoteDIvoire),
            Self::Kaapverdische => Some(CountryCode::CaboVerde),
            Self::Kameroense => Some(CountryCode::Cameroon),
            Self::Kenyaanse => Some(CountryCode::Kenya),
            // Zaire was renamed Democratic Republic of the Congo on 15-07-1997.
            Self::Zairese => Some(CountryCode::DemocraticRepublicOfTheCongo),
            Self::Lesothaanse => Some(CountryCode::Lesotho),
            Self::Liberiaanse => Some(CountryCode::Liberia),
            Self::Libische => Some(CountryCode::Libya),
            Self::Malagassische => Some(CountryCode::Madagascar),
            Self::Malawische => Some(CountryCode::Malawi),
            Self::Malinese => Some(CountryCode::Mali),
            Self::Marokkaanse => Some(CountryCode::Morocco),
            Self::Mauritaanse => Some(CountryCode::Mauritania),
            Self::Mauritiaanse => Some(CountryCode::Mauritius),
            Self::Mozambikaanse => Some(CountryCode::Mozambique),
            // Swaziland was renamed Eswatini on 19-04-2018.
            Self::Swazische => Some(CountryCode::Eswatini),
            Self::Nigerese => Some(CountryCode::Niger),
            Self::Nigeriaanse => Some(CountryCode::Nigeria),
            Self::Ugandese => Some(CountryCode::Uganda),
            Self::GuineeBissause => Some(CountryCode::GuineaBissau),
            Self::ZuidAfrikaanse => Some(CountryCode::SouthAfrica),
            Self::Eswatinische => Some(CountryCode::Eswatini),
            Self::Zimbabwaanse => Some(CountryCode::Zimbabwe),
            Self::Rwandese => Some(CountryCode::Rwanda),
            Self::BurgerVanSaoTomeEnPrincipe => Some(CountryCode::SaoTomeAndPrincipe),
            Self::Senegalese => Some(CountryCode::Senegal),
            Self::SierraLeoonse => Some(CountryCode::SierraLeone),
            Self::Soedanese => Some(CountryCode::Sudan),
            Self::Somalische => Some(CountryCode::Somalia),
            Self::Tanzaniaanse => Some(CountryCode::Tanzania),
            Self::Togolese => Some(CountryCode::Togo),
            Self::Tsjadische => Some(CountryCode::Chad),
            Self::Tunesische => Some(CountryCode::Tunisia),
            Self::Zambiaanse => Some(CountryCode::Zambia),
            Self::ZuidSoedanese => Some(CountryCode::SouthSudan),
            Self::Bahamaanse => Some(CountryCode::Bahamas),
            Self::Belizaanse => Some(CountryCode::Belize),
            Self::Canadese => Some(CountryCode::Canada),
            Self::CostaRicaanse => Some(CountryCode::CostaRica),
            Self::Cubaanse => Some(CountryCode::Cuba),
            Self::Dominicaanse => Some(CountryCode::DominicanRepublic),
            Self::Salvadoraanse => Some(CountryCode::ElSalvador),
            Self::Guatemalaanse => Some(CountryCode::Guatemala),
            Self::Haitiaanse => Some(CountryCode::Haiti),
            Self::Hondurese => Some(CountryCode::Honduras),
            Self::Jamaicaanse => Some(CountryCode::Jamaica),
            Self::Mexicaanse => Some(CountryCode::Mexico),
            Self::Nicaraguaanse => Some(CountryCode::Nicaragua),
            Self::Panamese => Some(CountryCode::Panama),
            Self::BurgerVanTrinidadEnTobago => Some(CountryCode::TrinidadAndTobago),
            Self::AmerikaansBurger => Some(CountryCode::UnitedStates),
            Self::Argentijnse => Some(CountryCode::Argentina),
            Self::Barbadaanse => Some(CountryCode::Barbados),
            Self::Boliviaanse => Some(CountryCode::Bolivia),
            Self::Braziliaanse => Some(CountryCode::Brazil),
            Self::Chileense => Some(CountryCode::Chile),
            Self::Colombiaanse => Some(CountryCode::Colombia),
            Self::Ecuadoraanse => Some(CountryCode::Ecuador),
            Self::Guyaanse => Some(CountryCode::Guyana),
            Self::Paraguayaanse => Some(CountryCode::Paraguay),
            Self::Peruaanse => Some(CountryCode::Peru),
            Self::Surinaamse => Some(CountryCode::Suriname),
            Self::Uruguayaanse => Some(CountryCode::Uruguay),
            Self::Venezolaanse => Some(CountryCode::Venezuela),
            Self::Grenadaanse => Some(CountryCode::Grenada),
            Self::BurgerVanSaintKittsEnNevis => Some(CountryCode::SaintKittsAndNevis),
            Self::Afghaanse => Some(CountryCode::Afghanistan),
            Self::Bahreinse => Some(CountryCode::Bahrain),
            Self::Bhutaanse => Some(CountryCode::Bhutan),
            // Burma was renamed to Myanmar on 18-06-1989.
            Self::Burmaanse => Some(CountryCode::Myanmar),
            Self::Bruneise => Some(CountryCode::BruneiDarussalam),
            Self::Cambodjaanse => Some(CountryCode::Cambodia),
            Self::SriLankaanse => Some(CountryCode::SriLanka),
            Self::Chinese => Some(CountryCode::China),
            Self::Cyprische => Some(CountryCode::Cyprus),
            Self::Filipijnse => Some(CountryCode::Philippines),
            Self::Taiwanese => Some(CountryCode::Taiwan),
            Self::Indiase => Some(CountryCode::India),
            Self::Indonesische => Some(CountryCode::Indonesia),
            Self::Iraakse => Some(CountryCode::Iraq),
            Self::Iraanse => Some(CountryCode::Iran),
            Self::Israelische => Some(CountryCode::Israel),
            Self::Japanse => Some(CountryCode::Japan),
            // North Yemen merged into Yemen on 22-05-1990.
            Self::NoordJemenitische => Some(CountryCode::Yemen),
            Self::Jordaanse => Some(CountryCode::Jordan),
            Self::Koeweitse => Some(CountryCode::Kuwait),
            Self::Laotiaanse => Some(CountryCode::Laos),
            Self::Libanese => Some(CountryCode::Lebanon),
            Self::Maldivische => Some(CountryCode::Maldives),
            Self::Maleisische => Some(CountryCode::Malaysia),
            Self::Mongolische => Some(CountryCode::Mongolia),
            Self::Omaanse => Some(CountryCode::Oman),
            Self::Nepalese => Some(CountryCode::Nepal),
            Self::NoordKoreaanse => Some(CountryCode::NorthKorea),
            Self::Pakistaanse => Some(CountryCode::Pakistan),
            Self::Qatarese => Some(CountryCode::Qatar),
            Self::SaoediArabische => Some(CountryCode::SaudiArabia),
            Self::Singaporese => Some(CountryCode::Singapore),
            Self::Syrische => Some(CountryCode::Syria),
            Self::Thaise => Some(CountryCode::Thailand),
            Self::BurgerVanDeVerenigdeArabischeEmiraten => Some(CountryCode::UnitedArabEmirates),
            Self::Turkse => Some(CountryCode::Turkiye),
            // South Yemen merged into Yemen on 22-05-1990.
            Self::ZuidJemenitische => Some(CountryCode::Yemen),
            Self::ZuidKoreaanse => Some(CountryCode::SouthKorea),
            Self::Vietnamese => Some(CountryCode::Vietnam),
            Self::Bengalese => Some(CountryCode::Bangladesh),
            Self::Australische => Some(CountryCode::Australia),
            Self::PapoeaNieuwGuinese => Some(CountryCode::PapuaNewGuinea),
            Self::NieuwZeelandse => Some(CountryCode::NewZealand),
            // Western Samoa was renamed Samoa on 04-07-1997.
            Self::WestSamoaanse => Some(CountryCode::Samoa),
            Self::Samoaanse => Some(CountryCode::Samoa),
            Self::BurgerVanAntiguaEnBarbuda => Some(CountryCode::AntiguaAndBarbuda),
            Self::Vanuatuaanse => Some(CountryCode::Vanuatu),
            Self::Fijische => Some(CountryCode::Fiji),
            // A British-nationality-law status.
            Self::BurgerVanBritseAfhankelijkeGebieden => Some(CountryCode::UnitedKingdom),
            Self::Tongaanse => Some(CountryCode::Tonga),
            Self::Nauruaanse => Some(CountryCode::Nauru),
            Self::Palause => Some(CountryCode::Palau),
            // A US-nationality-law status.
            Self::AmerikaansOnderdaan => Some(CountryCode::UnitedStates),
            Self::Salomonseilandse => Some(CountryCode::SolomonIslands),
            Self::Micronesische => Some(CountryCode::Micronesia),
            Self::Seychelse => Some(CountryCode::Seychelles),
            Self::Kiribatische => Some(CountryCode::Kiribati),
            Self::Tuvaluaanse => Some(CountryCode::Tuvalu),
            Self::SaintLuciaanse => Some(CountryCode::SaintLucia),
            Self::BurgerVanDominica => Some(CountryCode::Dominica),
            Self::BurgerVanSaintVincentEnDeGrenadines => Some(CountryCode::SaintVincentAndTheGrenadines),
            // A British-nationality-law status.
            Self::BritishNationalOverseas => Some(CountryCode::UnitedKingdom),
            Self::BurgerVanDemocratischeRepubliekCongo => Some(CountryCode::DemocraticRepublicOfTheCongo),
            Self::BurgerVanTimorLeste => Some(CountryCode::TimorLeste),
            // Serbia and Montenegro split into Serbia and Montenegro on 03-06-2006.
            Self::BurgerVanServieEnMontenegro => None,
            Self::Servische => Some(CountryCode::Serbia),
            Self::Montenegrijnse => Some(CountryCode::Montenegro),
            Self::Staatloos => None,
            Self::VastgesteldNietNederlander => None,
            Self::Overig => None,
        }
    }

    /// The value to put in the `nationalities` attribute: the mapped alpha-2 country, the Rulebook's own special value
    /// for `Self::Staatloos` ("QS", stateless), the widely used placeholder for `Self::Kosovaarse` ("XK") or any other
    /// unmapped variant ("QU", unknown).
    fn as_attribute(&self) -> String {
        match self.alpha2() {
            Some(country) => country.to_string(),
            None if matches!(self, Self::Staatloos) => "QS".to_owned(),
            None if matches!(self, Self::Kosovaarse) => "XK".to_owned(),
            None => "QU".to_owned(),
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

        format!("{}{}{}", self.house_number, house_letter, house_number_addition)
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
    fn should_handle_house_number(#[case] json_file_name: &str, #[case] expected_house_number: &str) {
        let brp_persons: BrpPersons = serde_json::from_str(&read_json(json_file_name)).unwrap();
        let brp_person = brp_persons.persons.first().unwrap();
        assert_eq!(expected_house_number, brp_person.residence.address.locator_designator());
    }

    #[test]
    fn should_convert_brp_person_to_issuable_vec() {
        let mut brp_persons: BrpPersons = serde_json::from_str(&read_json("frouke")).unwrap();
        let issuer_metadata = PidIssuerMetadata {
            issuing_country: "NL".to_owned(),
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
                "age_over_18": {
                    "type": "bool",
                    "value": true
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
                            "value": "Nederland"
                        }
                    }
                }
            }),
            serde_json::to_value(pid_card).unwrap()
        );
    }
}
