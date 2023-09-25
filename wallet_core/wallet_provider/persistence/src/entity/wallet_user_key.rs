//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "wallet_user_key")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub wallet_user_id: Uuid,
    #[sea_orm(unique)]
    pub identifier: String,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub private_key_der: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::wallet_user::Entity",
        from = "Column::WalletUserId",
        to = "super::wallet_user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    WalletUser,
}

impl Related<super::wallet_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WalletUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}