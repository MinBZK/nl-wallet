use chrono::{DateTime, Local};
use entity::event_log::{EventStatus, EventType};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletEvent {
    pub(crate) event_type: EventType,
    pub(crate) timestamp: DateTime<Local>,
    pub(crate) remote_party_certificate: Option<Vec<u8>>, // TODO Is there a better type to use for certificate?
    pub(crate) status: EventStatus,
}

impl WalletEvent {
    pub fn new(
        event_type: EventType,
        timestamp: DateTime<Local>,
        remote_party_certificate: Option<Vec<u8>>,
        status: EventStatus,
    ) -> Self {
        Self {
            event_type,
            timestamp,
            remote_party_certificate,
            status,
        }
    }
}
