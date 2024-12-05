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
                    .table(Mdoc::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Mdoc::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Mdoc::DocType).text().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(MdocCopy::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MdocCopy::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(MdocCopy::DisclosureCount)
                            .unsigned()
                            .default(0)
                            .not_null(),
                    )
                    .col(ColumnDef::new(MdocCopy::MdocId).uuid().not_null())
                    .col(ColumnDef::new(MdocCopy::Mdoc).binary().not_null())
                    // In sqlite/sqlcipher foreign keys can only be created as part of the create table statement.
                    .foreign_key(
                        ForeignKey::create()
                            .from(MdocCopy::Table, MdocCopy::MdocId)
                            .to(Mdoc::Table, Mdoc::Id)
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
            .drop_table(Table::drop().table(MdocCopy::Table).to_owned())
            .await?;

        manager.drop_table(Table::drop().table(Mdoc::Table).to_owned()).await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum Mdoc {
    Table,
    Id,
    DocType,
}

#[derive(Iden)]
enum MdocCopy {
    Table,
    Id,
    DisclosureCount,
    MdocId,
    Mdoc,
}
