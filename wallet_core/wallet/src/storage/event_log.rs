use chrono::{DateTime, Utc};
use entity::event_log::EventType;

use nl_wallet_mdoc::utils::x509::Certificate;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Success,
    Error(String),
    Cancelled,
}

impl From<Status> for entity::event_log::EventStatus {
    fn from(source: Status) -> Self {
        match source {
            Status::Success => Self::Success,
            Status::Error(_) => Self::Error,
            Status::Cancelled => Self::Cancelled,
        }
    }
}

impl Status {
    pub fn description(&self) -> Option<String> {
        if let Status::Error(description) = self {
            Some(description.to_owned())
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletEvent {
    pub(crate) event_type: EventType,
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) remote_party_certificate: Certificate,
    pub(crate) status: Status,
}

impl WalletEvent {
    pub fn new(
        event_type: EventType,
        timestamp: DateTime<Utc>,
        remote_party_certificate: Certificate,
        status: Status,
    ) -> Self {
        Self {
            event_type,
            timestamp,
            remote_party_certificate,
            status,
        }
    }
}
