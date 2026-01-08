use chrono::DateTime;
use chrono::Utc;
use rand::Rng;

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

        // If the attestation is revoked, we don't issue expiration notifications
        if attestation
            .validity
            .revocation_status
            .is_some_and(|revocation_status| revocation_status == RevocationStatus::Revoked)
        {
            return None;
        }

        let status = ValidityStatus::from_window(&attestation.validity.validity_window, time);

        match status {
            ValidityStatus::Expired => Some(vec_nonempty![Notification {
                id: rand::thread_rng().r#gen(),
                typ: NotificationType::Expired { attestation },
                targets: vec_nonempty![DisplayTarget::Dashboard],
            }]),
            ValidityStatus::ExpiresSoon { .. } => {
                let until = attestation.validity.validity_window.valid_until.unwrap();
                Some(vec_nonempty![
                    Notification {
                        id: rand::thread_rng().r#gen(),
                        typ: NotificationType::ExpiresSoon {
                            attestation: attestation.clone(),
                            expires_at: until,
                        },
                        targets: vec_nonempty![DisplayTarget::Dashboard],
                    },
                    Notification {
                        id: rand::thread_rng().r#gen(),
                        typ: NotificationType::Expired { attestation },
                        targets: vec_nonempty![DisplayTarget::Os { notify_at: until }],
                    },
                ])
            }
            ValidityStatus::Valid {
                notify_at,
                expires: Some(until),
            } => {
                let mut notes = vec![Notification {
                    id: rand::thread_rng().r#gen(),
                    typ: NotificationType::Expired {
                        attestation: attestation.clone(),
                    },
                    targets: vec_nonempty![DisplayTarget::Os { notify_at: until }],
                }];

                if let Some(soon_notify_at) = notify_at {
                    notes.push(Notification {
                        id: rand::thread_rng().r#gen(),
                        typ: NotificationType::ExpiresSoon {
                            attestation,
                            expires_at: until,
                        },
                        targets: vec_nonempty![DisplayTarget::Os {
                            notify_at: soon_notify_at
                        }],
                    });
                }

                Some(VecNonEmpty::try_from(notes).unwrap())
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
        // Expires in 10 days (Outside the 7-day threshold)
        let expiry = now + Duration::days(10);
        presentation.validity.validity_window = ValidityWindow {
            valid_from: Some(now - Duration::days(1)),
            valid_until: Some(expiry),
        };

        // Get the expected notification date from the source of truth
        let expected_notify_at = match ValidityStatus::from_window(&presentation.validity.validity_window, now) {
            ValidityStatus::Valid {
                notify_at: Some(at), ..
            } => at,
            _ => panic!("Expected status to be Valid with a notify_at date"),
        };

        let notifications = Notification::create_for_attestation(presentation, &generator);
        let ns = notifications.expect("Expected notifications");

        // Should have 2 notifications: one for Expiration and one for ExpiresSoon
        assert_eq!(ns.len().get(), 2);

        // Find the "ExpiresSoon" notification specifically (it's at index 1 in the current impl)
        let soon_note = ns
            .iter()
            .find(|n| matches!(n.typ, NotificationType::ExpiresSoon { .. }))
            .expect("Expected an ExpiresSoon notification");

        // Must have Os target scheduled for the future, and NO Dashboard target yet
        assert_matches!(soon_note.targets[0], DisplayTarget::Os { notify_at } if notify_at == expected_notify_at);
        assert!(!soon_note.targets.iter().any(|t| matches!(t, DisplayTarget::Dashboard)));

        // Find the "Expired" notification specifically
        let expired_note = ns
            .iter()
            .find(|n| matches!(n.typ, NotificationType::Expired { .. }))
            .expect("Expected an Expired notification");
        assert_matches!(expired_note.targets[0], DisplayTarget::Os { notify_at } if notify_at == expiry);
    }
}
