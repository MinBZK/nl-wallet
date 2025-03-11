use itertools::Itertools;

use super::attestation::Attestation;
use super::disclosure::DisclosureType;
use super::disclosure::Organization;
use super::disclosure::RPLocalizedStrings;
use super::disclosure::RequestPolicy;
use super::localize::LocalizedString;

pub struct WalletEvents(Vec<WalletEvent>);

pub enum WalletEvent {
    Disclosure {
        // ISO8601
        date_time: String,
        relying_party: Organization,
        purpose: Vec<LocalizedString>,
        requested_attestations: Option<Vec<Attestation>>,
        request_policy: RequestPolicy,
        status: DisclosureStatus,
        typ: DisclosureType,
    },
    Issuance {
        // ISO8601
        date_time: String,
        attestation: Attestation,
    },
}

pub enum DisclosureStatus {
    Success,
    Cancelled,
    Error,
}

impl IntoIterator for WalletEvents {
    type Item = WalletEvent;
    type IntoIter = std::vec::IntoIter<WalletEvent>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<wallet::WalletEvent> for WalletEvents {
    fn from(source: wallet::WalletEvent) -> Self {
        let result = match source {
            wallet::WalletEvent::Issuance {
                attestations,
                timestamp,
                ..
            } => attestations
                .into_iter()
                .map(|attestation| WalletEvent::Issuance {
                    date_time: timestamp.to_rfc3339(),
                    attestation: attestation.into(),
                })
                .collect(),
            wallet::WalletEvent::Disclosure {
                attestations,
                timestamp,
                reader_registration,
                status,
                r#type,
                ..
            } => {
                let reader_registration = *reader_registration;
                let request_policy = RequestPolicy::from(&reader_registration);
                let attestations = attestations.into_iter().map(Attestation::from).collect_vec();

                vec![WalletEvent::Disclosure {
                    date_time: timestamp.to_rfc3339(),
                    relying_party: Organization::from(reader_registration.organization),
                    purpose: RPLocalizedStrings(reader_registration.purpose_statement).into(),
                    request_policy,
                    requested_attestations: (!attestations.is_empty()).then_some(attestations),
                    status: status.into(),
                    typ: r#type.into(),
                }]
            }
        };
        WalletEvents(result)
    }
}

impl From<wallet::DisclosureStatus> for DisclosureStatus {
    fn from(source: wallet::DisclosureStatus) -> Self {
        match source {
            wallet::DisclosureStatus::Success => DisclosureStatus::Success,
            wallet::DisclosureStatus::Cancelled => DisclosureStatus::Cancelled,
            wallet::DisclosureStatus::Error => DisclosureStatus::Error,
        }
    }
}
