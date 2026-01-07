use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use rand::Rng;

use attestation_data::validity::ValidityWindow;
use token_status_list::verification::verifier::RevocationStatus;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::AttestationPresentation;

const EXPIRES_SOON_WINDOW: Duration = Duration::days(7);

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

        let ValidityWindow {
            valid_from,
            valid_until,
        } = attestation.validity.validity_window;

        // If the attestation is not yet valid, we don't issue expiration notifications and assume this will be
        // scheduled later.
        if valid_from.is_some_and(|from| time < from) {
            return None;
        }

        // If the attestation is revoked, we don't issue expiration notifications
        if attestation
            .validity
            .revocation_status
            .is_some_and(|revocation_status| revocation_status == RevocationStatus::Revoked)
        {
            return None;
        }

        // Determine which notifications should be issued based on the validity window
        match valid_until {
            // Show an expired notification only on the dashboard
            Some(until) if time > until => Some(vec_nonempty![Notification {
                id: rand::thread_rng().r#gen(),
                typ: NotificationType::Expired { attestation },
                targets: vec_nonempty![DisplayTarget::Dashboard],
            }]),
            // Always schedule expiration notifications for valid attestations
            Some(until) if time <= until => {
                // If we're currently within the window, only schedule expiration notification on the dashboard
                let expires_soon_targets = if time > until - EXPIRES_SOON_WINDOW {
                    vec_nonempty![DisplayTarget::Dashboard]
                } else {
                    vec_nonempty![DisplayTarget::Os {
                        notify_at: until - EXPIRES_SOON_WINDOW,
                    }]
                };

                // A valid attestation should always have a notification scheduled for when it expires, as well as an
                // expires soon notification.
                Some(vec_nonempty![
                    Notification {
                        id: rand::thread_rng().r#gen(),
                        typ: NotificationType::ExpiresSoon {
                            attestation: attestation.clone(),
                            expires_at: until,
                        },
                        targets: expires_soon_targets,
                    },
                    Notification {
                        id: rand::thread_rng().r#gen(),
                        typ: NotificationType::Expired { attestation },
                        targets: vec_nonempty![DisplayTarget::Os { notify_at: until }],
                    },
                ])
            }
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
    fn test_notification_not_yet_valid() {
        let now = Utc::now();
        let generator = MockTimeGenerator::new(now);

        let mut presentation = AttestationPresentation::new_mock();
        presentation.validity.validity_window = ValidityWindow {
            valid_from: Some(now + Duration::days(1)),
            valid_until: Some(now + Duration::days(365)),
        };

        let notifications = Notification::create_for_attestation(presentation, &generator);
        assert!(notifications.is_none());
    }

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

        let notification = Notification::create_for_attestation(presentation, &generator);

        // Should be None because revoked cards don't get expiration notifications
        assert!(notification.is_none());
    }

    #[test]
    fn test_notification_expires_soon_within_window_uses_dashboard() {
        let now = Utc::now();
        let generator = MockTimeGenerator::new(now);

        let mut presentation = AttestationPresentation::new_mock();
        // Expires in 5 days (Inside the 7-day EXPIRES_SOON_WINDOW)
        let expiry = now + Duration::days(5);
        presentation.validity.validity_window = ValidityWindow {
            valid_from: Some(now - Duration::days(1)),
            valid_until: Some(expiry),
        };

        let notifications = Notification::create_for_attestation(presentation, &generator);
        let ns = notifications.expect("Expected notifications");

        assert_eq!(ns.len().get(), 2);

        // 1. ExpiresSoon entry
        let soon_note = &ns[0];
        assert_matches!(soon_note.typ, NotificationType::ExpiresSoon { expires_at, .. } if expires_at == expiry);
        // Must have Dashboard target, and MUST NOT have Os target for "soon" because we are already in the window
        assert!(soon_note.targets.iter().any(|t| matches!(t, DisplayTarget::Dashboard)));
        assert!(!soon_note.targets.iter().any(|t| matches!(t, DisplayTarget::Os { .. })));

        // 2. Scheduled Expired entry (always Os when currently valid)
        let expired_note = &ns[1];
        assert_matches!(expired_note.typ, NotificationType::Expired { .. });
        assert_matches!(expired_note.targets[0], DisplayTarget::Os { notify_at } if notify_at == expiry);
    }

    #[test]
    fn test_notification_expires_soon_outside_window_uses_os() {
        let now = Utc::now();
        let generator = MockTimeGenerator::new(now);

        let mut presentation = AttestationPresentation::new_mock();
        // Expires in 10 days (Outside the 7-day EXPIRES_SOON_WINDOW)
        let expiry = now + Duration::days(10);
        presentation.validity.validity_window = ValidityWindow {
            valid_from: Some(now - Duration::days(1)),
            valid_until: Some(expiry),
        };

        let notifications = Notification::create_for_attestation(presentation, &generator);
        let ns = notifications.expect("Expected notifications");

        assert_eq!(ns.len().get(), 2);

        // 1. ExpiresSoon entry
        let soon_note = &ns[0];
        assert_matches!(soon_note.typ, NotificationType::ExpiresSoon { .. });
        // Must have Os target scheduled for the future, and NO Dashboard target yet
        assert_matches!(soon_note.targets[0], DisplayTarget::Os { notify_at } if notify_at == expiry - EXPIRES_SOON_WINDOW);
        assert!(!soon_note.targets.iter().any(|t| matches!(t, DisplayTarget::Dashboard)));

        // 2. Scheduled Expired entry
        assert_matches!(ns[1].typ, NotificationType::Expired { .. });
        assert_matches!(ns[1].targets[0], DisplayTarget::Os { notify_at } if notify_at == expiry);
    }
}
