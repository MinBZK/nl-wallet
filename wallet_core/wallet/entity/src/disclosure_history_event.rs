use chrono::DateTime;
use chrono::Utc;
use sea_orm::entity::prelude::*;

use crate::disclosure_history_event_attestation_type;
use crate::history_attestation_type;

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
#[sea_orm(table_name = "disclosure_history_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub relying_party_certificate: Vec<u8>,
    pub status: EventStatus,
    pub attestations: Option<Json>,
    pub r#type: EventType,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<history_attestation_type::Entity> for Entity {
    fn to() -> RelationDef {
        disclosure_history_event_attestation_type::Relation::HistoryAttestationType.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            disclosure_history_event_attestation_type::Relation::HistoryEvent
                .def()
                .rev(),
        )
    }
}
