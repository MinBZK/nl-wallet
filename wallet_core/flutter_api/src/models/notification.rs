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
        expires_at: String, // ISO8601
    },
}

pub struct AppNotification {
    pub id: u32,
    pub typ: NotificationType,
    pub targets: Vec<DisplayTarget>,
}

impl From<wallet::Notification> for AppNotification {
    fn from(value: wallet::Notification) -> Self {
        AppNotification {
            id: value.id,
            typ: value.typ.into(),
            targets: vec![],
        }
    }
}

impl From<wallet::NotificationType> for NotificationType {
    fn from(value: wallet::NotificationType) -> Self {
        match value {
            wallet::NotificationType::CardExpired { card } => NotificationType::CardExpired { card: card.into() },
            wallet::NotificationType::CardExpiresSoon { card, expires_at } => NotificationType::CardExpiresSoon {
                card: card.into(),
                expires_at: expires_at.to_rfc3339(),
            },
        }
    }
}
