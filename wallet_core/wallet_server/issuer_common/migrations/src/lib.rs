use async_trait::async_trait;
use sea_orm_migration::prelude::*;

mod m20220101_000001_create_proof_nonce_table;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = vec![];

        migrations.extend(server_utils_migrations::Migrator::migrations());
        migrations.extend(status_lists_migrations::Migrator::migrations());

        migrations.push(Box::new(m20220101_000001_create_proof_nonce_table::Migration));

        migrations
    }
}
