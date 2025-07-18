use sea_orm::entity::prelude::*;

use super::attestation;
use super::disclosure_event;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "disclosure_event_attestation")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub disclosure_event_id: Uuid,
    pub attestation_id: Option<Uuid>,
    pub attestation_presentation: Json,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    DisclosureEvent,
    Attestation,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::DisclosureEvent => Entity::belongs_to(disclosure_event::Entity)
                .from(Column::DisclosureEventId)
                .to(disclosure_event::Column::Id)
                .on_delete(ForeignKeyAction::NoAction)
                .into(),
            Self::Attestation => Entity::belongs_to(attestation::Entity)
                .from(Column::AttestationId)
                .to(attestation::Column::Id)
                .on_delete(ForeignKeyAction::SetNull)
                .into(),
        }
    }
}

impl Related<disclosure_event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DisclosureEvent.def()
    }
}

impl Related<attestation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attestation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
