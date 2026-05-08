use async_trait::async_trait;
use sea_orm_migration::prelude::*;

mod m20220101_000001_create_proof_nonce_table;
mod m20260507_000001_create_pushed_authorization_request_table;

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

/// Migrations for the OpenID4VCI Authorization Phase tables.
///
/// Only consumers that implement the Authorization Phase need to apply these on
/// top of the shared [`Migrator`].
pub fn authorization_phase_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(
        m20260507_000001_create_pushed_authorization_request_table::Migration,
    )]
}
