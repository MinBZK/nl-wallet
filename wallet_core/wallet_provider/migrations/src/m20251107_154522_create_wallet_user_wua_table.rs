use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use crate::m20250102_000010_create_wallet_user_table::WalletUser;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WalletUserWua::Table)
                    .col(uuid(WalletUserWua::WuaId).primary_key())
                    .col(uuid(WalletUserWua::WalletUserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wallet_user_wua_wallet_user_id")
                            .from(WalletUserWua::Table, WalletUserWua::WalletUserId)
                            .to(WalletUser::Table, WalletUser::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum WalletUserWua {
    Table,
    WuaId,
    WalletUserId,
}
