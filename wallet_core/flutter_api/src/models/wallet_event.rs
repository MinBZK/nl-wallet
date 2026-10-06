use itertools::Itertools;

use super::attestation::AttestationPresentation;
use super::disclosure::DisclosureType;
use super::disclosure::RequestPolicy;
use super::disclosure::request_purpose;
use super::localize::LocalizedString;
use super::organization::Organization;

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
    Deletion {
        id: String,
        // ISO8601
        date_time: String,
        attestation: AttestationPresentation,
    },
}

pub enum DisclosureStatus {
    Success,
    Cancelled,
    Error,
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
                organization,
                status,
                r#type,
                ..
            } => {
                let attestations = attestations
                    .into_iter()
                    .map(AttestationPresentation::from)
                    .collect_vec();

                WalletEvent::Disclosure {
                    id: id.to_string(),
                    date_time: timestamp.to_rfc3339(),
                    purpose: request_purpose(&organization),
                    request_policy: RequestPolicy::new(organization.privacy_policy_url.as_ref()),
                    relying_party: (*organization).into(),
                    shared_attestations: (!attestations.is_empty()).then_some(attestations),
                    status: status.into(),
                    typ: r#type.into(),
                }
            }
            wallet::WalletEvent::Deletion {
                id,
                timestamp,
                attestation,
            } => WalletEvent::Deletion {
                id: id.to_string(),
                date_time: timestamp.to_rfc3339(),
                attestation: (*attestation).into(),
            },
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

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;
    use wallet::attestation_data::Organization;

    use super::WalletEvent;

    #[rstest]
    #[case::registered(Some("Verify your identity"), "Verify your identity")]
    #[case::proximity(None, "Disclosure")]
    fn disclosure_history_registration_fields(
        #[case] registered_purpose: Option<&str>,
        #[case] expected: &str,
        #[values(None, Some("https://example.org/privacy"))] privacy_policy_url: Option<&str>,
    ) {
        let mut stored = serde_json::to_value(Organization::default()).unwrap();
        if let Some(purpose) = registered_purpose {
            stored["purpose"] = json!([{ "lang": "en", "value": purpose }]);
        }
        if let Some(url) = privacy_policy_url {
            stored["privacyPolicyUrl"] = json!(url);
        }
        let event = WalletEvent::from(wallet::WalletEvent::Disclosure {
            id: Default::default(),
            timestamp: Default::default(),
            organization: Box::new(serde_json::from_value(stored).unwrap()),
            attestations: vec![],
            status: wallet::DisclosureStatus::Success,
            r#type: wallet::attestation_data::DisclosureType::Regular,
        });
        let WalletEvent::Disclosure {
            purpose,
            request_policy,
            ..
        } = event
        else {
            panic!("expected disclosure event");
        };
        assert_eq!(purpose[0].language, "en");
        assert_eq!(purpose[0].value, expected);
        assert_eq!(request_policy.policy_url.as_deref(), privacy_policy_url);
    }
}
