use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

use crate::{history_doc_type, issuance_history_event_doc_type};

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "issuance_history_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub attributes: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<history_doc_type::Entity> for Entity {
    fn to() -> RelationDef {
        issuance_history_event_doc_type::Relation::HistoryDocType.def()
    }

    fn via() -> Option<RelationDef> {
        Some(issuance_history_event_doc_type::Relation::HistoryEvent.def().rev())
    }
}
