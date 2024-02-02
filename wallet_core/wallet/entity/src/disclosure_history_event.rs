use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

use crate::{disclosure_history_event_doc_type, history_doc_type};

#[derive(Clone, Debug, Eq, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum EventStatus {
    #[sea_orm(string_value = "Success")]
    Success,
    #[sea_orm(string_value = "Error")]
    Error,
    #[sea_orm(string_value = "Cancelled")]
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "disclosure_history_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub relying_party_certificate: Vec<u8>,
    pub status: EventStatus,
    // TODO: How to translate a generic description? Shouldn't this be part of the audit log?
    pub status_description: Option<String>,
    pub attributes: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<history_doc_type::Entity> for Entity {
    fn to() -> RelationDef {
        disclosure_history_event_doc_type::Relation::HistoryDocType.def()
    }

    fn via() -> Option<RelationDef> {
        Some(disclosure_history_event_doc_type::Relation::HistoryEvent.def().rev())
    }
}
