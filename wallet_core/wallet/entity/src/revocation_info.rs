use sea_orm::FromQueryResult;
use uuid::Uuid;

use super::attestation::AttestationFormat;

#[derive(Clone, Debug, Eq, PartialEq, FromQueryResult)]
pub struct RevocationInfo {
    pub attestation_copy_id: Uuid,
    pub attestation_format: AttestationFormat,
    pub status_list_url: String,
    pub status_list_index: u32,
}
