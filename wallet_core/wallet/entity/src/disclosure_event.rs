use chrono::DateTime;
use chrono::Utc;
use sea_orm::entity::prelude::*;

use super::disclosure_event_attestation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum EventStatus {
    #[sea_orm(string_value = "Success")]
    Success,
    #[sea_orm(string_value = "Error")]
    Error,
    #[sea_orm(string_value = "Cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum EventType {
    #[sea_orm(string_value = "Login")]
    Login,
    #[sea_orm(string_value = "Regular")]
    Regular,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "disclosure_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub relying_party_certificate: Vec<u8>,
    pub status: EventStatus,
    pub r#type: EventType,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    DisclosureEventAttestation,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::DisclosureEventAttestation => Entity::has_many(disclosure_event_attestation::Entity).into(),
        }
    }
}

impl Related<disclosure_event_attestation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DisclosureEventAttestation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
