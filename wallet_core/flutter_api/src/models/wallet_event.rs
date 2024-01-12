use wallet::{EventStatus, HistoryEvent};

use crate::models::{
    card::{Card, LocalizedString},
    disclosure::{Organization, RPLocalizedStrings, RequestPolicy, RequestedCard},
};

pub enum WalletEvent {
    Disclosure {
        //ISO8601
        date_time: String,
        relying_party: Organization,
        purpose: Vec<LocalizedString>,
        requested_cards: Option<Vec<RequestedCard>>,
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

pub struct WalletEvents(Vec<WalletEvent>);

impl IntoIterator for WalletEvents {
    type Item = WalletEvent;
    type IntoIter = std::vec::IntoIter<WalletEvent>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<HistoryEvent> for WalletEvents {
    fn from(source: HistoryEvent) -> Self {
        let result = match source {
            HistoryEvent::Issuance { timestamp, mdocs } => {
                mdocs
                    .into_iter()
                    .map(|mdoc| {
                        WalletEvent::Issuance {
                            date_time: timestamp.to_rfc3339(),
                            issuer: pid_issuer_organization(), // TODO: read from certificate
                            card: mdoc.into(),
                        }
                    })
                    .collect()
            }
            HistoryEvent::Disclosure {
                status,
                timestamp,
                reader_registration,
                attributes,
            } => {
                let reader_registration = *reader_registration;
                vec![WalletEvent::Disclosure {
                    date_time: timestamp.to_rfc3339(),
                    request_policy: RequestPolicy::from(&reader_registration),
                    relying_party: Organization::from(reader_registration.organization),
                    purpose: RPLocalizedStrings(reader_registration.purpose_statement).into(),
                    requested_cards: attributes.map(|mdocs| mdocs.into_iter().map(RequestedCard::from).collect()),
                    status: status.into(),
                }]
            }
        };
        WalletEvents(result)
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
