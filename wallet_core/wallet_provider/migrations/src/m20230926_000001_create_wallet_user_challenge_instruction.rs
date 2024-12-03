use async_trait::async_trait;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use crate::m20230616_000001_create_wallet_user_table::WalletUser;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WalletUserInstructionChallenge::Table)
                    .col(pk_uuid(WalletUserInstructionChallenge::Id))
                    .col(uuid(WalletUserInstructionChallenge::WalletUserId))
                    .col(binary(WalletUserInstructionChallenge::InstructionChallenge))
                    .col(timestamp_with_time_zone(
                        WalletUserInstructionChallenge::ExpirationDateTime,
                    ))
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
                            .name("wallet_user_instruction_challenge_unique_wallet_user_id")
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
