use sea_orm::entity::prelude::*;
use uuid::Uuid;

use crypto::x509::DistinguishedName;

use crate::compressed_blob::CompressedBlob;

use super::attestation;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "attestation_copy")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub disclosure_count: u32,
    pub attestation_id: Uuid,
    pub key_identifier: String,
    pub status_list_url: Option<String>,
    pub status_list_index: Option<u32>,
    pub issuer_certificate_dn: DistinguishedName,
    pub revocation_status: Option<String>,
    pub attestation_format: AttestationFormat,
    pub attestation: CompressedBlob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum AttestationFormat {
    #[sea_orm(string_value = "dc+sd-jwt")]
    SdJwt,
    #[sea_orm(string_value = "mso_mdoc")]
    Mdoc,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AttestationType,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AttestationType => Entity::belongs_to(attestation::Entity)
                .from(Column::AttestationId)
                .to(attestation::Column::Id)
                .into(),
        }
    }
}
impl Related<attestation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AttestationType.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
