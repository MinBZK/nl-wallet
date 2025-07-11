use itertools::Itertools;

use super::attestation::AttestationPresentation;
use super::disclosure::DisclosureType;
use super::disclosure::Organization;
use super::disclosure::RPLocalizedStrings;
use super::disclosure::RequestPolicy;
use super::localize::LocalizedString;

pub struct WalletEvents(Vec<WalletEvent>);

pub enum WalletEvent {
    Disclosure {
        id: String,
        // ISO8601
        date_time: String,
        relying_party: Organization,
        purpose: Vec<LocalizedString>,
        shared_attestations: Option<Vec<AttestationPresentation>>,
        request_policy: RequestPolicy,
        status: DisclosureStatus,
        typ: DisclosureType,
    },
    Issuance {
        id: String,
        // ISO8601
        date_time: String,
        attestation: AttestationPresentation,
        renewed: bool,
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

impl From<wallet::WalletEvent> for WalletEvent {
    fn from(source: wallet::WalletEvent) -> Self {
        match source {
            wallet::WalletEvent::Issuance {
                id,
                attestation,
                timestamp,
                renewed,
                ..
            } => WalletEvent::Issuance {
                id: id.to_string(),
                date_time: timestamp.to_rfc3339(),
                attestation: (*attestation).into(),
                renewed,
            },
            wallet::WalletEvent::Disclosure {
                id,
                attestations,
                timestamp,
                reader_registration,
                status,
                r#type,
                ..
            } => {
                let reader_registration = *reader_registration;
                let request_policy = RequestPolicy::from(&reader_registration);
                let attestations = attestations
                    .into_iter()
                    .map(AttestationPresentation::from)
                    .collect_vec();

                WalletEvent::Disclosure {
                    id: id.to_string(),
                    date_time: timestamp.to_rfc3339(),
                    relying_party: Organization::from(reader_registration.organization),
                    purpose: RPLocalizedStrings(reader_registration.purpose_statement).into(),
                    request_policy,
                    shared_attestations: (!attestations.is_empty()).then_some(attestations),
                    status: status.into(),
                    typ: r#type.into(),
                }
            }
        }
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
