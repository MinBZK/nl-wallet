use std::fmt::Display;
use std::fmt::Formatter;

use indexmap::IndexMap;
use quick_xml::DeError;
use quick_xml::NsReader;
use quick_xml::events::Event;
use quick_xml::name::Namespace;
use quick_xml::name::ResolveResult::Bound;
use serde::Deserialize;
use serde::Deserializer;

use crate::gba::error::Error;

const LO3_NAMESPACE: &[u8] = b"http://www.bprbzk.nl/GBA/LO3/version1.1";
const LRD_NAMESPACE: &[u8] = b"http://www.bprbzk.nl/GBA/LRDPlus/version1.1";
const CATEGORIEVOORKOMENS_TAG: &str = "categorievoorkomens";
const RESULTAAT_TAG: &str = "resultaat";

const EMPTY_RESULT_CODE: &str = "33";
const EMPTY_RESULT_LETTER: &str = "G";

fn parse_response_xml(xml: &str) -> Result<GbaResponse, Error> {
    let mut reader = NsReader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut voorkomens_xml = Vec::new();
    let mut result_xml: Option<String> = None;

    // Since quick_xml doesn't handle deserializing nested identical tags well, we'll first take the
    // 'categorievoorkomens' tag using event based parsing. After that, we can safely deserialize using Serde.
    loop {
        match reader.read_event() {
            Err(e) => return Err(Error::XmlDeserialization(DeError::InvalidXml(e))),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let (ns, local) = reader.resolve_element(e.name());
                if ns == Bound(Namespace(LO3_NAMESPACE)) && local.as_ref() == CATEGORIEVOORKOMENS_TAG.as_bytes() {
                    let end = e.to_end();
                    let text = reader.read_text(end.name())?;
                    voorkomens_xml.push(text);
                } else if ns == Bound(Namespace(LRD_NAMESPACE)) && local.as_ref() == RESULTAAT_TAG.as_bytes() {
                    let end = e.to_end();
                    let text = reader.read_text(end.name())?;
                    result_xml = Some(format!("<{0}>{1}</{0}>", RESULTAAT_TAG, &text));
                }
            }
            _ => (),
        }
    }

    let result: GbaResult = quick_xml::de::from_str(&result_xml.ok_or(Error::UnexpectedResponse)?)?;

    let categorievoorkomens = voorkomens_xml
        .iter()
        .map(|voorkomen_xml| Ok(quick_xml::de::from_str(voorkomen_xml)?))
        .collect::<Result<Vec<Categorievoorkomen>, Error>>()?;

    let response = GbaResponse {
        categorievoorkomens,
        result,
    };
    Ok(response)
}

#[derive(Clone)]
pub struct GbaResponse {
    pub result: GbaResult,
    pub categorievoorkomens: Vec<Categorievoorkomen>,
}

impl GbaResponse {
    pub fn new(xml: &str) -> Result<Self, Error> {
        parse_response_xml(xml)
    }

    pub fn empty() -> Self {
        Self {
            result: GbaResult::empty(),
            categorievoorkomens: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.result.is_empty() && self.categorievoorkomens.is_empty()
    }

    pub fn get_mandatory_voorkomen(&self, category_number: u8) -> Result<&Categorievoorkomen, Error> {
        self.categorievoorkomens
            .iter()
            .find(|voorkomen| voorkomen.categorienummer == category_number)
            .ok_or(Error::MissingCategorie(category_number))
    }

    pub fn get_voorkomens(&self, category_number: u8) -> Vec<&Categorievoorkomen> {
        self.categorievoorkomens
            .iter()
            .filter(|voorkomen| voorkomen.categorienummer == category_number)
            .collect()
    }

    pub fn is_error(&self) -> bool {
        self.result.letter != "A" && self.result.letter != "G"
    }

    pub fn as_error(&self) -> Result<(), Error> {
        if self.is_error() {
            Err(Error::GbaErrorResponse(self.result.clone()))
        } else {
            Ok(())
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct GbaResult {
    pub code: String,

    pub letter: String,

    #[serde(rename = "omschrijving")]
    pub description: String,

    #[serde(rename = "referentie")]
    pub reference: String,
}

impl GbaResult {
    pub fn empty() -> Self {
        Self {
            code: String::from(EMPTY_RESULT_CODE),
            letter: String::from(EMPTY_RESULT_LETTER),
            description: String::from("Geen gegevens gevonden."),
            reference: String::from("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.code.as_str() == EMPTY_RESULT_CODE && self.letter.as_str() == EMPTY_RESULT_LETTER
    }
}

impl Display for GbaResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "foutcode: {}{:0>3}, description: {}, reference: {}",
            self.letter, self.code, self.description, self.reference
        )
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Categorievoorkomen {
    pub categorienummer: u8,

    #[serde(deserialize_with = "unwrap_list")]
    pub elementen: Elementen,
}

#[derive(Clone, Debug)]
pub struct Elementen {
    pub map: IndexMap<u16, String>,
}

impl Elementen {
    pub fn get_mandatory(&self, element_number: u16) -> Result<String, Error> {
        self.map
            .get(&element_number)
            .cloned()
            .ok_or(Error::MissingElement(element_number))
    }

    pub fn get_optional(&self, element_number: u16) -> Option<String> {
        self.map.get(&element_number).cloned()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    nummer: u16,
    waarde: String,
}

fn unwrap_list<'de, D>(deserializer: D) -> Result<Elementen, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Items {
        #[serde(default)]
        item: Vec<Item>,
    }

    let items = Items::deserialize(deserializer)?;
    let mut map = IndexMap::new();
    items.item.into_iter().for_each(|item| {
        map.insert(item.nummer, item.waarde);
    });
    Ok(Elementen { map })
}
