use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::disclosure_type::DisclosureType;
use crypto::x509::BorrowingCertificate;
use entity::disclosure_event::EventStatus;

use crate::attestation::AttestationPresentation;

pub type DisclosureStatus = EventStatus;

#[derive(Debug, Clone, Copy)]
pub enum DataDisclosureStatus {
    Disclosed,
    NotDisclosed,
}

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
        // TODO (PVW-4135): Only store reader registration in event.
        reader_certificate: Box<BorrowingCertificate>,
        reader_registration: Box<ReaderRegistration>,
        status: DisclosureStatus,
        r#type: DisclosureType,
    },
}

impl WalletEvent {
    pub fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            Self::Issuance { timestamp, .. } => timestamp,
            Self::Disclosure { timestamp, .. } => timestamp,
        }
    }
}
