use sea_orm::entity::prelude::*;

use super::attestation;
use super::issuance_event;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "issuance_event_attestation")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub issuance_event_id: Uuid,
    pub attestation_id: Option<Uuid>,
    pub attestation_presentation: Json,
    pub renewed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    IssuanceEvent,
    Attestation,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::IssuanceEvent => Entity::belongs_to(issuance_event::Entity)
                .from(Column::IssuanceEventId)
                .to(issuance_event::Column::Id)
                .into(),
            Self::Attestation => Entity::belongs_to(attestation::Entity)
                .from(Column::AttestationId)
                .to(attestation::Column::Id)
                .into(),
        }
    }
}

impl Related<issuance_event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IssuanceEvent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
