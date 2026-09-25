use sea_orm_migration::prelude::*;

mod m20260113_000001_create_audit_log;
mod m20260925_000000_add_user_id_to_audit_log;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260113_000001_create_audit_log::Migration),
            Box::new(m20260925_000000_add_user_id_to_audit_log::Migration),
        ]
    }
}
