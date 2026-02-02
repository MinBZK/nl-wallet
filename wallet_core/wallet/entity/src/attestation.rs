use chrono::DateTime;
use chrono::Utc;
use derive_more::Constructor;
use sea_orm::ActiveModelBehavior;
use sea_orm::DeriveEntityModel;
use sea_orm::DerivePrimaryKey;
use sea_orm::EntityTrait;
use sea_orm::EnumIter;
use sea_orm::FromJsonQueryResult;
use sea_orm::PrimaryKeyTrait;
use sea_orm::Related;
use sea_orm::RelationDef;
use sea_orm::RelationTrait;
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
    #[sea_orm(column_name = "expiration_date_time")]
    pub expiration: Option<DateTime<Utc>>,
    #[sea_orm(column_name = "not_before_date_time")]
    pub not_before: Option<DateTime<Utc>>,
    pub extended_types: ExtendedTypesModel,
    pub type_metadata: TypeMetadataModel,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, Constructor)]
#[serde(transparent)]
pub struct ExtendedTypesModel {
    pub attestation_types: Vec<String>,
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
