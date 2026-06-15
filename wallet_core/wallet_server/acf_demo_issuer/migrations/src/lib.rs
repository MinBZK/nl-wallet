use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = issuer_common_migrations::Migrator::migrations();
        migrations.extend(issuer_common_migrations::authorization_phase_migrations());
        migrations
    }
}
