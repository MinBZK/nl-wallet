use indexmap::IndexMap;
use quick_xml::{
    de::from_str,
    events::Event,
    name::{Namespace, ResolveResult::Bound},
    DeError, NsReader,
};
use serde::{Deserialize, Deserializer};

use crate::gba::error::Error;

const GBA_NAMESPACE: &[u8] = b"http://www.bprbzk.nl/GBA/LO3/version1.1";
const CATEGORIEVOORKOMENS_TAG: &[u8] = b"categorievoorkomens";

pub fn parse_xml(xml: &str) -> Result<Vec<Categorievoorkomen>, Error> {
    let mut reader = NsReader::from_str(xml);
    reader.trim_text(true);

    let mut categorievoorkomens = Vec::new();

    // Since quick_xml doesn't handle deserializing nested identical tags well, we'll first take the
    // 'categorievoorkomens' tag using event based parsing. After that, we can safely deserialize using Serde.
    loop {
        match reader.read_event() {
            Err(e) => return Err(Error::XmlDeserialization(DeError::InvalidXml(e))),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let (ns, local) = reader.resolve_element(e.name());
                if ns == Bound(Namespace(GBA_NAMESPACE)) && local.as_ref() == CATEGORIEVOORKOMENS_TAG {
                    let end = e.to_end();
                    let text = reader.read_text(end.name())?;
                    categorievoorkomens.push(text);
                }
            }
            _ => (),
        }
    }

    let result = categorievoorkomens
        .iter()
        .map(|categorievoorkomen| Ok(from_str(categorievoorkomen)?))
        .collect::<Result<Vec<Categorievoorkomen>, Error>>()?;

    Ok(result)
}

#[derive(Clone)]
pub struct GbaResponse {
    pub categorievoorkomens: Vec<Categorievoorkomen>,
}

impl GbaResponse {
    pub fn new(xml: &str) -> Result<Self, Error> {
        Ok(Self {
            categorievoorkomens: parse_xml(xml)?,
        })
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
}

#[derive(Deserialize, Clone, Debug)]
pub struct Categorievoorkomen {
    pub categorienummer: u8,

    #[serde(deserialize_with = "unwrap_list")]
    pub elementen: Elementen,
}

#[derive(Clone, Debug)]
pub struct Elementen {
    pub map: IndexMap<String, String>,
}

impl Elementen {
    pub fn get_mandatory(&self, element_number: &str) -> Result<String, Error> {
        self.map
            .get(element_number)
            .cloned()
            .ok_or(Error::MissingElement(String::from(element_number)))
    }

    pub fn get_optional(&self, element_number: &str) -> Option<String> {
        self.map.get(element_number).cloned()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    nummer: String,
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
