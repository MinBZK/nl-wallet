use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

use crate::{history_doc_type, history_event_doc_type};

#[derive(Clone, Debug, Eq, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum EventType {
    #[sea_orm(string_value = "Issuance")]
    Issuance,
    #[sea_orm(string_value = "Disclosure")]
    Disclosure,
}

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
#[sea_orm(table_name = "history_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "type")]
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub remote_party_certificate: Vec<u8>,
    pub status: EventStatus,
    // TODO How to translate a generic description? Shouldn't this be part of the audit log?
    pub status_description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<history_doc_type::Entity> for Entity {
    fn to() -> RelationDef {
        history_event_doc_type::Relation::HistoryDocType.def()
    }

    fn via() -> Option<RelationDef> {
        Some(history_event_doc_type::Relation::HistoryEvent.def().rev())
    }
}
