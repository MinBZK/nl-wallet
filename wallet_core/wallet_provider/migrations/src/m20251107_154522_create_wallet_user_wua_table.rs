use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use status_lists_migrations::m20250925_000002_create_attestation_batch::AttestationBatch;

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
                    .col(big_integer(WalletUserWua::Id).primary_key().auto_increment())
                    .col(big_integer(WalletUserWua::WuaId))
                    .col(uuid(WalletUserWua::WalletUserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wallet_user_wua_batch_id")
                            .from(WalletUserWua::Table, WalletUserWua::WuaId)
                            .to(AttestationBatch::Table, AttestationBatch::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
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
    Id,
    WuaId,
    WalletUserId,
}
