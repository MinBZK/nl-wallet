use sea_orm::FromQueryResult;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

use crypto::x509::DistinguishedName;

use super::attestation_copy;

#[derive(Clone, Debug, Eq, PartialEq, FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "attestation_copy::Entity")]
pub struct RevocationInfo {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub status_list_url: Option<String>,
    pub status_list_index: Option<u32>,
    pub issuer_certificate_dn: DistinguishedName,
}
