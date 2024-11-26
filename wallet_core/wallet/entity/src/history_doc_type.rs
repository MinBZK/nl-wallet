use sea_orm::entity::prelude::*;

use crate::disclosure_history_event;
use crate::disclosure_history_event_doc_type;
use crate::issuance_history_event;
use crate::issuance_history_event_doc_type;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "history_doc_type")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub doc_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<disclosure_history_event::Entity> for Entity {
    fn to() -> RelationDef {
        disclosure_history_event_doc_type::Relation::HistoryEvent.def()
    }

    fn via() -> Option<RelationDef> {
        Some(disclosure_history_event_doc_type::Relation::HistoryDocType.def().rev())
    }
}

impl Related<issuance_history_event::Entity> for Entity {
    fn to() -> RelationDef {
        issuance_history_event_doc_type::Relation::HistoryEvent.def()
    }

    fn via() -> Option<RelationDef> {
        Some(issuance_history_event_doc_type::Relation::HistoryDocType.def().rev())
    }
}
