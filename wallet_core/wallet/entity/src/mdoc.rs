use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mdoc")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub doc_type: String,
    pub type_metadata: Json,
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
