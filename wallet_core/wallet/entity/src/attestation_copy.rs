use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "attestation_copy")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub disclosure_count: u32,
    pub attestation_id: Uuid,
    pub attestation_format: AttestationFormat,
    pub attestation: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum AttestationFormat {
    #[sea_orm(string_value = "dc+sd-jwt")]
    SdJwt,
    #[sea_orm(string_value = "mso_mdoc")]
    Mdoc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::attestation::Entity",
        from = "Column::AttestationId",
        to = "super::attestation::Column::Id"
    )]
    AttestationType,
}

impl Related<super::attestation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AttestationType.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
