use wallet::{self, AttributeValue, Document, GenderAttributeValue};

pub struct Card {
    pub id: Option<String>,
    pub doc_type: String,
    pub attributes: Vec<CardAttribute>,
}

pub struct CardAttribute {
    pub key: String,
    pub labels: Vec<LocalizedString>,
    pub value: CardValue,
}

pub enum CardValue {
    String { value: String },
    Boolean { value: bool },
    Date { value: String },
    Gender { value: GenderCardValue },
}

pub enum GenderCardValue {
    Unknown,
    Male,
    Female,
    NotApplicable,
}

impl From<GenderAttributeValue> for GenderCardValue {
    fn from(value: GenderAttributeValue) -> Self {
        match value {
            GenderAttributeValue::Unknown => Self::Unknown,
            GenderAttributeValue::Male => Self::Male,
            GenderAttributeValue::Female => Self::Female,
            GenderAttributeValue::NotApplicable => Self::NotApplicable,
        }
    }
}

pub struct LocalizedString {
    pub language: String,
    pub value: String,
}

impl From<Document> for Card {
    fn from(value: Document) -> Self {
        let attributes = value
            .attributes
            .into_iter()
            .map(|(key, attribute)| {
                let labels = attribute
                    .key_labels
                    .into_iter()
                    .map(|(language, value)| LocalizedString {
                        language: language.to_string(),
                        value: value.to_string(),
                    })
                    .collect();

                CardAttribute {
                    key: key.to_string(),
                    labels,
                    value: CardValue::from(attribute.value),
                }
            })
            .collect();

        Card {
            id: value.id,
            doc_type: value.doc_type.to_string(),
            attributes,
        }
    }
}

impl From<AttributeValue> for CardValue {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::String(s) => Self::String { value: s },
            AttributeValue::Boolean(b) => Self::Boolean { value: b },
            AttributeValue::Date(d) => Self::Date {
                value: d.format("%Y-%m-%d").to_string(),
            },
            AttributeValue::Gender(g) => Self::Gender { value: g.into() },
        }
    }
}

impl From<(String, String)> for LocalizedString {
    fn from(value: (String, String)) -> Self {
        LocalizedString {
            language: value.0,
            value: value.1,
        }
    }
}

pub fn mock_cards() -> Vec<Card> {
    vec![
        Card {
            id: "025b9338-a1f7-4c57-bdaa-9992be55e5f0".to_string().into(),
            doc_type: "pid_id".to_string(),
            attributes: vec![
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "First names".to_string()).into(),
                        ("nl".to_string(), "Voornamen".to_string()).into(),
                    ],
                    key: "pid.firstNames".to_string(),
                    value: CardValue::String {
                        value: "Willeke Liselotte".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Last name".to_string()).into(),
                        ("nl".to_string(), "Achternaam".to_string()).into(),
                    ],
                    key: "pid.lastName".to_string(),
                    value: CardValue::String {
                        value: "De Bruijn".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Birthname".to_string()).into(),
                        ("nl".to_string(), "Geboortenaam".to_string()).into(),
                    ],
                    key: "pid.birthName".to_string(),
                    value: CardValue::String {
                        value: "Molenaar".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Sex".to_string()).into(),
                        ("nl".to_string(), "Geslacht".to_string()).into(),
                    ],
                    key: "pid.sex".to_string(),
                    value: CardValue::String {
                        value: "Vrouw".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Birthdate".to_string()).into(),
                        ("nl".to_string(), "Geboortedatum".to_string()).into(),
                    ],
                    key: "pid.birthDate".to_string(),
                    value: CardValue::Date {
                        value: "1997-05-10".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Older than 18".to_string()).into(),
                        ("nl".to_string(), "Ouder dan 18".to_string()).into(),
                    ],
                    key: "pid.olderThan18".to_string(),
                    value: CardValue::Boolean { value: true },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Birthplace".to_string()).into(),
                        ("nl".to_string(), "Geboorteplaats".to_string()).into(),
                    ],
                    key: "pid.birthplace".to_string(),
                    value: CardValue::String {
                        value: "Delft".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Country of birth".to_string()).into(),
                        ("nl".to_string(), "Geboorteland".to_string()).into(),
                    ],
                    key: "pid.countryOfBirth".to_string(),
                    value: CardValue::String {
                        value: "Nederland".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Citizen service number (BSN)".to_string()).into(),
                        ("nl".to_string(), "Burgerservicenummer (BSN)".to_string()).into(),
                    ],
                    key: "pid.bsn".to_string(),
                    value: CardValue::String {
                        value: "999999999".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Nationality".to_string()).into(),
                        ("nl".to_string(), "Nationaliteit".to_string()).into(),
                    ],
                    key: "pid.nationality".to_string(),
                    value: CardValue::String {
                        value: "Nederlands".to_string(),
                    },
                },
            ],
        },
        Card {
            id: "f553eb44-13a2-416c-aa9d-61a3f75b029a".to_string().into(),
            doc_type: "pid_address".to_string(),
            attributes: vec![
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Street".to_string()).into(),
                        ("nl".to_string(), "Straatnaam".to_string()).into(),
                    ],
                    key: "pid.streetName".to_string(),
                    value: CardValue::String {
                        value: "Turfmarkt".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "House number".to_string()).into(),
                        ("nl".to_string(), "Huisnummer".to_string()).into(),
                    ],
                    key: "pid.houseNumber".to_string(),
                    value: CardValue::String {
                        value: "147".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Postal code".to_string()).into(),
                        ("nl".to_string(), "Postcode".to_string()).into(),
                    ],
                    key: "pid.postalCode".to_string(),
                    value: CardValue::String {
                        value: "2511 DP".to_string(),
                    },
                },
                CardAttribute {
                    labels: vec![
                        ("en".to_string(), "Residence".to_string()).into(),
                        ("nl".to_string(), "Woonplaats".to_string()).into(),
                    ],
                    key: "pid.residence".to_string(),
                    value: CardValue::String {
                        value: "Den Haag".to_string(),
                    },
                },
            ],
        },
    ]
}
