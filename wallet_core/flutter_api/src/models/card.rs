pub struct Card {
    pub id: i64,
    pub doc_type: String,
    pub issuer: String,
    pub attributes: Vec<CardAttribute>,
}

pub struct CardAttribute {
    pub key: String,
    pub labels: Vec<LocalizedString>,
    pub value: CardValue,
}

pub enum CardValue {
    String { value: String },
    Integer { value: i64 },
    Double { value: f64 },
    Boolean { value: bool },
    Date { value: String },
}

pub struct LocalizedString {
    pub language: String,
    pub value: String,
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
            id: 0,
            doc_type: "pid_id".to_string(),
            issuer: "rvig".to_string(),
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
                    value: CardValue::Integer { value: 999999999 },
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
            id: 1,
            doc_type: "pid_address".to_string(),
            issuer: "rvig".to_string(),
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
