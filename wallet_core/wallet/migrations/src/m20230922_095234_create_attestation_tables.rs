use async_trait::async_trait;
use entity::attestation_copy::AttestationFormat;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Iterable;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Attestation::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Attestation::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Attestation::Type).text().not_null())
                    .col(ColumnDef::new(Attestation::TypeMetadata).json().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AttestationCopy::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AttestationCopy::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(AttestationCopy::DisclosureCount)
                            .unsigned()
                            .default(0)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AttestationCopy::AttestationId).uuid().not_null())
                    .col(
                        ColumnDef::new(AttestationCopy::Format)
                            // SQLite doesn't have proper enum support, so we simulate that here with a custom type
                            // (SQLite is dynamically typed) and a check expression.
                            .custom(AttestationCopy::EnumText)
                            .check(Expr::col(AttestationCopy::Format).is_in(AttestationFormat::iter()))
                            .not_null(),
                    )
                    .col(ColumnDef::new(AttestationCopy::Attestation).binary().not_null())
                    // In sqlite/sqlcipher foreign keys can only be created as part of the create table statement.
                    .foreign_key(
                        ForeignKey::create()
                            .from(AttestationCopy::Table, AttestationCopy::AttestationId)
                            .to(Attestation::Table, Attestation::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(AttestationCopy::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Attestation::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Attestation {
    Table,
    Id,
    #[sea_orm(iden = "attestation_type")]
    Type,
    TypeMetadata,
}

#[derive(DeriveIden)]
enum AttestationCopy {
    Table,
    Id,
    DisclosureCount,
    AttestationId,
    #[sea_orm(iden = "attestation_format")]
    Format,
    Attestation,
    EnumText,
}
