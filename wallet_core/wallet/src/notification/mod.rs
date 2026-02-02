use chrono::DateTime;
use chrono::Utc;
use rand::random;

use token_status_list::verification::verifier::RevocationStatus;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::AttestationPresentation;
use crate::ValidityStatus;

#[derive(Debug)]
pub struct Notification {
    pub id: i32,
    pub typ: NotificationType,
    pub targets: VecNonEmpty<DisplayTarget>,
}

#[derive(Debug)]
pub enum NotificationType {
    Expired {
        attestation: AttestationPresentation,
    },
    ExpiresSoon {
        attestation: AttestationPresentation,
        expires_at: DateTime<Utc>,
    },
    Revoked {
        attestation: AttestationPresentation,
    },
}

#[derive(Debug)]
pub enum DisplayTarget {
    Os { notify_at: DateTime<Utc> },
    Dashboard,
}

impl Notification {
    pub fn create_for_attestation(
        attestation: AttestationPresentation,
        time_generator: &impl Generator<DateTime<Utc>>,
    ) -> Option<VecNonEmpty<Self>> {
        let time = time_generator.generate();

        // If the attestation is revoked, only issue a revocation notification to the dashboard
        if attestation
            .validity
            .revocation_status
            .is_some_and(|revocation_status| revocation_status == RevocationStatus::Revoked)
        {
            return Some(vec_nonempty![Notification {
                id: random(),
                typ: NotificationType::Revoked { attestation },
                targets: vec_nonempty![DisplayTarget::Dashboard]
            }]);
        }

        let status = ValidityStatus::from_window(&attestation.validity.validity_window, time);

        match status {
            ValidityStatus::Expired { .. } => Some(vec_nonempty![Notification {
                id: random(),
                typ: NotificationType::Expired { attestation },
                targets: vec_nonempty![DisplayTarget::Dashboard],
            }]),
            ValidityStatus::ExpiresSoon { expires_at, .. } => Some(vec_nonempty![
                Notification {
                    id: random(),
                    typ: NotificationType::ExpiresSoon {
                        attestation: attestation.clone(),
                        expires_at,
                    },
                    targets: vec_nonempty![DisplayTarget::Dashboard],
                },
                Notification {
                    id: random(),
                    typ: NotificationType::Expired { attestation },
                    targets: vec_nonempty![DisplayTarget::Os { notify_at: expires_at }],
                },
            ]),
            ValidityStatus::ValidUntil { notify_at, expires_at } => Some(vec_nonempty![
                Notification {
                    id: random(),
                    typ: NotificationType::ExpiresSoon {
                        attestation: attestation.clone(),
                        expires_at,
                    },
                    targets: vec_nonempty![DisplayTarget::Os { notify_at }],
                },
                Notification {
                    id: random(),
                    typ: NotificationType::Expired { attestation },
                    targets: vec_nonempty![DisplayTarget::Os { notify_at: expires_at }],
                },
            ]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use chrono::Duration;
    use chrono::Utc;

    use attestation_data::validity::ValidityWindow;
    use utils::generator::mock::MockTimeGenerator;

    use super::*;

    #[test]
    fn test_notification_expired() {
        let now = Utc::now();
        let generator = MockTimeGenerator::new(now);

        let mut presentation = AttestationPresentation::new_mock();
        presentation.validity.validity_window = ValidityWindow {
            valid_from: Some(now - Duration::days(1)),
            valid_until: Some(now - Duration::days(1)),
        };

        let notifications = Notification::create_for_attestation(presentation, &generator);
        let ns = notifications.expect("Expected notification");

        assert_eq!(ns.len().get(), 1);
        assert_matches!(ns[0].typ, NotificationType::Expired { .. });
        assert!(ns[0].targets.iter().any(|t| matches!(t, DisplayTarget::Dashboard)));
    }

    #[test]
    fn test_notification_revoked() {
        let now = Utc::now();
        let generator = MockTimeGenerator::new(now);

        let mut presentation = AttestationPresentation::new_mock();
        // Card is technically valid and expires soon, BUT it is revoked
        presentation.validity.revocation_status = Some(RevocationStatus::Revoked);
        presentation.validity.validity_window = ValidityWindow {
            valid_from: Some(now - Duration::days(1)),
            valid_until: Some(now + Duration::days(1)),
        };

        let notifications = Notification::create_for_attestation(presentation, &generator);
        let ns = notifications.expect("Expected notifications");

        let revoked = ns
            .iter()
            .find(|n| matches!(n.typ, NotificationType::Revoked { .. }))
            .expect("Expected a Revoked notification");
        assert!(revoked.targets.iter().all(|t| matches!(t, DisplayTarget::Dashboard)));
    }

    #[test]
    fn test_notification_expires_soon_within_window_uses_dashboard() {
        let now = Utc::now();
        let generator = MockTimeGenerator::new(now);

        let mut presentation = AttestationPresentation::new_mock();
        // Expires in 5 days (Inside the 7-day threshold)
        let expiry = now + Duration::days(5);
        presentation.validity.validity_window = ValidityWindow {
            valid_from: Some(now - Duration::days(1)),
            valid_until: Some(expiry),
        };

        let notifications = Notification::create_for_attestation(presentation, &generator);
        let ns = notifications.expect("Expected notifications");

        assert_eq!(ns.len().get(), 2);

        // 1. Find the ExpiresSoon entry - Should trigger Dashboard immediately
        let soon = ns
            .iter()
            .find(|n| matches!(n.typ, NotificationType::ExpiresSoon { .. }))
            .expect("Expected an ExpiresSoon notification");
        assert!(soon.targets.iter().any(|t| matches!(t, DisplayTarget::Dashboard)));

        // 2. Find the Expired entry - Should be scheduled for the actual expiry date
        let expired = ns
            .iter()
            .find(|n| matches!(n.typ, NotificationType::Expired { .. }))
            .expect("Expected an Expired notification");

        // This was the failing line: it must match the expiry date exactly
        assert_matches!(expired.targets[0], DisplayTarget::Os { notify_at } if notify_at == expiry);
    }

    #[test]
    fn test_notification_expires_soon_outside_window_uses_os() {
        let now = Utc::now();
        let generator = MockTimeGenerator::new(now);

        let mut presentation = AttestationPresentation::new_mock();
        // Expires in 10 days (Outside the 7-day threshold)
        let expiry = now + Duration::days(10);
        presentation.validity.validity_window = ValidityWindow {
            valid_from: Some(now - Duration::days(1)),
            valid_until: Some(expiry),
        };

        // Get the expected notification date from the source of truth
        let expected_notify_at = match ValidityStatus::from_window(&presentation.validity.validity_window, now) {
            ValidityStatus::ValidUntil { notify_at, .. } => notify_at,
            _ => panic!("Expected status to be Valid with a notify_at date"),
        };

        let notifications = Notification::create_for_attestation(presentation, &generator);
        let ns = notifications.expect("Expected notifications");

        // Should have 2 notifications: one for Expiration and one for ExpiresSoon
        assert_eq!(ns.len().get(), 2);

        // Find the "ExpiresSoon" notification specifically (it's at index 1 in the current impl)
        let soon = ns
            .iter()
            .find(|n| matches!(n.typ, NotificationType::ExpiresSoon { .. }))
            .expect("Expected an ExpiresSoon notification");

        // Must have Os target scheduled for the future, and NO Dashboard target yet
        assert_matches!(soon.targets[0], DisplayTarget::Os { notify_at } if notify_at == expected_notify_at);
        assert!(!soon.targets.iter().any(|t| matches!(t, DisplayTarget::Dashboard)));

        // Find the "Expired" notification specifically
        let expired = ns
            .iter()
            .find(|n| matches!(n.typ, NotificationType::Expired { .. }))
            .expect("Expected an Expired notification");
        assert_matches!(expired.targets[0], DisplayTarget::Os { notify_at } if notify_at == expiry);
    }
}
