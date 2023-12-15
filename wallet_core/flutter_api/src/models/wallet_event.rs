use wallet::{
    x509::{CertificateError, CertificateType},
    EventStatus,
};

use crate::models::{
    card::{Card, CardAttribute, CardPersistence, CardValue, LocalizedString},
    disclosure::{Organization, RPLocalizedStrings, RequestPolicy, RequestedCard},
};

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

impl WalletEvent {
    pub fn try_from(source: wallet::WalletEvent) -> Result<Vec<WalletEvent>, CertificateError> {
        let result = match source.event_type {
            wallet::EventType::Issuance(mdocs) => {
                mdocs
                    .0
                    .into_iter()
                    .map(|(doc_type, _)| {
                        WalletEvent::Issuance {
                            date_time: source.timestamp.to_rfc3339(),
                            // TODO How to properly detect PID issuer
                            issuer: pid_issuer_organization(),
                            // TODO extract from WalletEvent after event_log table stores mdoc
                            card: Card {
                                persistence: CardPersistence::InMemory,
                                doc_type,
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
                            },
                        }
                    })
                    .collect()
            }
            wallet::EventType::Disclosure(_) => {
                let certificate_type = CertificateType::from_certificate(&source.remote_party_certificate)?;
                let reader_registration = match certificate_type {
                    CertificateType::ReaderAuth(Some(reader_registration)) => Some(*reader_registration),
                    _ => None,
                }
                .unwrap();
                vec![WalletEvent::Disclosure {
                    date_time: source.timestamp.to_rfc3339(),
                    request_policy: RequestPolicy::from(&reader_registration),
                    relying_party: Organization::from(reader_registration.organization),
                    purpose: RPLocalizedStrings(reader_registration.purpose_statement).into(),
                    // TODO extract from WalletEvent after event_log table stores mdoc
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
                }]
            }
        };
        Ok(result)
    }
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

/// Hardcoded organization info for PID Issuer.
fn pid_issuer_organization() -> Organization {
    Organization {
        legal_name: vec![LocalizedString {
            language: "nl".to_owned(),
            value: "RvIG".to_owned(),
        }],
        display_name: vec![LocalizedString {
            language: "nl".to_owned(),
            value: "Rijksdienst voor Identiteitsgegevens".to_owned(),
        }],
        description: vec![LocalizedString {
            language: "nl".to_owned(),
            value: "Opvragen van PID (Person Identification Data)".to_owned(),
        }],
        category: vec![LocalizedString {
            language: "nl".to_owned(),
            value: "Overheid".to_owned(),
        }],
        image: None,
        web_url: Some("https://www.rvig.nl".to_owned()),
        kvk: Some(" 27373207".to_owned()),
        city: Some(vec![LocalizedString {
            language: "nl".to_owned(),
            value: "'s-Gravenhage".to_owned(),
        }]),
        department: None,
        country_code: Some("nl".to_owned()),
    }
}
