use chrono::{DateTime, Local};
use entity::transaction::{TransactionStatus, TransactionType};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionRecord {
    pub(crate) r#type: TransactionType,
    pub(crate) timestamp: DateTime<Local>,
    pub(crate) remote_party_certificate: Option<Vec<u8>>, // TODO Is there a better type to use for certificate?
    pub(crate) status: TransactionStatus,
}

impl TransactionRecord {
    pub fn new(
        r#type: TransactionType,
        timestamp: DateTime<Local>,
        remote_party_certificate: Option<Vec<u8>>,
        status: TransactionStatus,
    ) -> Self {
        Self {
            r#type,
            timestamp,
            remote_party_certificate,
            status,
        }
    }
}
