use crate::models::attestation::AttestationPresentation;

pub enum DisplayTarget {
    Os { notify_at: String }, // ISO8601
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
    Revoked {
        card: AttestationPresentation,
    },
}

pub struct AppNotification {
    pub id: i32,
    pub typ: NotificationType,
    pub targets: Vec<DisplayTarget>,
}

impl From<wallet::Notification> for AppNotification {
    fn from(value: wallet::Notification) -> Self {
        AppNotification {
            id: value.id,
            typ: value.typ.into(),
            targets: value.targets.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<wallet::NotificationType> for NotificationType {
    fn from(value: wallet::NotificationType) -> Self {
        match value {
            wallet::NotificationType::Expired { attestation } => NotificationType::CardExpired {
                card: attestation.into(),
            },
            wallet::NotificationType::ExpiresSoon {
                attestation,
                expires_at,
            } => NotificationType::CardExpiresSoon {
                card: attestation.into(),
                expires_at: expires_at.to_rfc3339(),
            },
            wallet::NotificationType::Revoked { attestation } => NotificationType::Revoked {
                card: attestation.into(),
            },
        }
    }
}

impl From<wallet::DisplayTarget> for DisplayTarget {
    fn from(value: wallet::DisplayTarget) -> Self {
        match value {
            wallet::DisplayTarget::Os { notify_at } => DisplayTarget::Os {
                notify_at: notify_at.to_rfc3339(),
            },
            wallet::DisplayTarget::Dashboard => DisplayTarget::Dashboard,
        }
    }
}
