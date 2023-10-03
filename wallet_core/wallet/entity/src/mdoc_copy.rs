use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mdoc_copy")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub mdoc_id: Uuid,
    pub mdoc: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::mdoc::Entity",
        from = "Column::MdocId",
        to = "super::mdoc::Column::Id"
    )]
    MdocType,
}

impl Related<super::mdoc::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MdocType.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
