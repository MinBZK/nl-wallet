use async_trait::async_trait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WalletUserInstructionChallenge::Table)
                    .col(
                        ColumnDef::new(WalletUserInstructionChallenge::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WalletUserInstructionChallenge::WalletUserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WalletUserInstructionChallenge::InstructionChallenge)
                            .binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WalletUserInstructionChallenge::ExpirationDateTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wallet_user_id")
                            .from(
                                WalletUserInstructionChallenge::Table,
                                WalletUserInstructionChallenge::WalletUserId,
                            )
                            .to(WalletUser::Table, WalletUser::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("uk_wallet_user_id")
                            .col(WalletUserInstructionChallenge::WalletUserId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum WalletUserInstructionChallenge {
    Table,
    Id,
    WalletUserId,
    InstructionChallenge,
    ExpirationDateTime,
}

#[derive(Iden)]
enum WalletUser {
    Table,
    Id,
}
