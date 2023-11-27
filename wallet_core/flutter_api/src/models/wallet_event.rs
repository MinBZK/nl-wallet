use wallet::{
    x509::{CertificateError, CertificateType},
    EventStatus,
};

use crate::models::card::{Card, CardAttribute, CardPersistence, CardValue, LocalizedString};
use crate::models::disclosure::{Organization, RPLocalizedStrings, RequestPolicy, RequestedCard};

pub enum WalletEvent {
    Disclosure {
        //ISO8601
        date_time: String,
        relying_party: Organization,
        purpose: Vec<LocalizedString>,
        requested_cards: Vec<RequestedCard>,
        request_policy: RequestPolicy,
        status: DisclosureStatus,
    },
    Issuance {
        //ISO8601
        date_time: String,
        issuer: Organization,
        card: Card,
    },
}

pub enum DisclosureStatus {
    Success,
    Cancelled,
    Error,
}

impl From<EventStatus> for DisclosureStatus {
    fn from(source: EventStatus) -> Self {
        match source {
            EventStatus::Success => DisclosureStatus::Success,
            EventStatus::Cancelled => DisclosureStatus::Cancelled,
            EventStatus::Error(_) => DisclosureStatus::Error,
        }
    }
}

impl TryFrom<wallet::WalletEvent> for WalletEvent {
    type Error = CertificateError;
    fn try_from(source: wallet::WalletEvent) -> Result<Self, Self::Error> {
        match source.event_type {
            wallet::EventType::Issuance => Ok(WalletEvent::Issuance {
                date_time: source.timestamp.to_rfc3339(),
                // issuer: Organization::from(reader_registration.organization),
                issuer: Organization {
                    legal_name: vec![LocalizedString {
                        language: "nl".to_owned(),
                        value: "RP Legal Name".to_owned(),
                    }],
                    display_name: vec![LocalizedString {
                        language: "nl".to_owned(),
                        value: "RP Display Name".to_owned(),
                    }],
                    description: vec![LocalizedString {
                        language: "nl".to_owned(),
                        value: "RP Description".to_owned(),
                    }],
                    image: None,
                    web_url: Some("https://example.org".to_owned()),
                    kvk: Some("1234 5678".to_owned()),
                    city: Some(vec![LocalizedString {
                        language: "nl".to_owned(),
                        value: "RP City".to_owned(),
                    }]),
                    country_code: Some("nl".to_owned()),
                },

                card: Card {
                    persistence: CardPersistence::InMemory,
                    doc_type: "com.example.pid".to_string(),
                    attributes: vec![],
                },
            }),
            wallet::EventType::Disclosure => {
                let certificate_type = CertificateType::from_certificate(&source.remote_party_certificate)?;
                let reader_registration = match certificate_type {
                    CertificateType::ReaderAuth(Some(reader_registration)) => Some(*reader_registration),
                    _ => None,
                }
                .unwrap();
                Ok(WalletEvent::Disclosure {
                    date_time: source.timestamp.to_rfc3339(),
                    request_policy: RequestPolicy::from(&reader_registration),
                    relying_party: Organization::from(reader_registration.organization),
                    purpose: RPLocalizedStrings(reader_registration.purpose_statement).into(),
                    requested_cards: vec![RequestedCard {
                        doc_type: "com.example.pid".to_string(),
                        attributes: vec![CardAttribute {
                            key: "sample".to_string(),
                            labels: vec![LocalizedString {
                                language: "en".to_string(),
                                value: "Sample label".to_string(),
                            }],
                            value: CardValue::String {
                                value: "Sample value".to_string(),
                            },
                        }],
                    }],
                    status: source.status.into(),
                })
            }
        }
    }
}
