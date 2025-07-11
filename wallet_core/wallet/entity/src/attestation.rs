use derive_more::Constructor;
use sea_orm::entity::prelude::*;
use sea_orm::FromJsonQueryResult;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;

use super::attestation_copy;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "attestation")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub attestation_type: String,
    pub type_metadata: TypeMetadataModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, Constructor)]
#[serde(transparent)]
pub struct TypeMetadataModel {
    pub documents: VerifiedTypeMetadataDocuments,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AttestationCopy,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AttestationCopy => Entity::has_many(attestation_copy::Entity).into(),
        }
    }
}

impl Related<attestation_copy::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AttestationCopy.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
