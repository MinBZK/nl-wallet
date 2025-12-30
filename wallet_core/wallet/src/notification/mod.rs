use chrono::DateTime;
use chrono::Utc;

use crate::AttestationPresentation;

pub struct Notification {
    pub id: u32,
    pub typ: NotificationType,
    // pub targets: Vec<DisplayTarget>,
}

pub enum NotificationType {
    CardExpired {
        card: AttestationPresentation,
    },
    CardExpiresSoon {
        card: AttestationPresentation,
        expires_at: DateTime<Utc>, // ISO8601
    },
}
