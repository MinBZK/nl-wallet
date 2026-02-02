use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = server_utils_migrations::Migrator::migrations();
        migrations.extend(status_lists_migrations::Migrator::migrations());
        migrations
    }
}
