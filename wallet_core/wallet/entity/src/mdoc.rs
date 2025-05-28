use derive_more::Constructor;
use sea_orm::entity::prelude::*;
use sea_orm::FromJsonQueryResult;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mdoc")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub doc_type: String,
    pub type_metadata: TypeMetadataModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, Constructor)]
#[serde(transparent)]
pub struct TypeMetadataModel {
    pub documents: VerifiedTypeMetadataDocuments,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::mdoc_copy::Entity")]
    MdocCopy,
}

impl Related<super::mdoc_copy::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MdocCopy.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
