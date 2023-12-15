use sea_orm::entity::prelude::*;

use crate::{history_event, history_event_doc_type};

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

impl Related<history_event::Entity> for Entity {
    fn to() -> RelationDef {
        history_event_doc_type::Relation::HistoryEvent.def()
    }

    fn via() -> Option<RelationDef> {
        Some(history_event_doc_type::Relation::HistoryDocType.def().rev())
    }
}
