use attestation_data::auth::Organization;
use attestation_data::disclosure_type::DisclosureType;
use chrono::DateTime;
use chrono::Utc;
use entity::disclosure_event::EventStatus;
use uuid::Uuid;

use crate::attestation::AttestationPresentation;

pub type DisclosureStatus = EventStatus;

#[derive(Debug, Clone, PartialEq)]
pub enum WalletEvent {
    Issuance {
        id: Uuid,
        attestation: Box<AttestationPresentation>,
        timestamp: DateTime<Utc>,
        renewed: bool,
    },
    Disclosure {
        id: Uuid,
        attestations: Vec<AttestationPresentation>,
        timestamp: DateTime<Utc>,
        organization: Box<Organization>,
        status: DisclosureStatus,
        r#type: DisclosureType,
    },
    Deletion {
        id: Uuid,
        timestamp: DateTime<Utc>,
        attestation: Box<AttestationPresentation>,
    },
}

impl WalletEvent {
    pub fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            Self::Issuance { timestamp, .. } => timestamp,
            Self::Disclosure { timestamp, .. } => timestamp,
            Self::Deletion { timestamp, .. } => timestamp,
        }
    }
}
