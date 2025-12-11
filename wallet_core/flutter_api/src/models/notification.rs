use crate::models::attestation::AttestationPresentation;

pub enum DisplayTarget {
    Os { notify_at: String /* ISO8601 */ },
    Dashboard,
}

pub enum NotificationType {
    CardExpired {
        card: AttestationPresentation,
    },
    CardExpiresSoon {
        card: AttestationPresentation,
        expires_at: String, /* ISO8601 */
    },
}

pub struct AppNotification {
    pub id: u32,
    pub typ: NotificationType,
    pub targets: Vec<DisplayTarget>,
}
