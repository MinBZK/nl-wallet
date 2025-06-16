use chrono::DateTime;
use chrono::Utc;
use sea_orm::entity::prelude::*;

use super::issuance_event_attestation;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "issuance_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    IssuanceEventAttestation,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::IssuanceEventAttestation => Entity::has_many(issuance_event_attestation::Entity).into(),
        }
    }
}

impl Related<issuance_event_attestation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IssuanceEventAttestation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
