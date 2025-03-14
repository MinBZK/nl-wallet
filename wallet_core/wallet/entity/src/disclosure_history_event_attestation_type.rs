use sea_orm::entity::prelude::*;

use crate::disclosure_history_event;
use crate::history_attestation_type;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "disclosure_history_event_attestation_type")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub disclosure_history_event_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub history_attestation_type_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    HistoryEvent,
    HistoryAttestationType,
}

impl ActiveModelBehavior for ActiveModel {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::HistoryEvent => Entity::belongs_to(disclosure_history_event::Entity)
                .from(Column::DisclosureHistoryEventId)
                .to(disclosure_history_event::Column::Id)
                .into(),
            Self::HistoryAttestationType => Entity::belongs_to(history_attestation_type::Entity)
                .from(Column::HistoryAttestationTypeId)
                .to(history_attestation_type::Column::Id)
                .into(),
        }
    }
}
