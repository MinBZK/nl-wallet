use sea_orm_migration::prelude::*;

mod m20250925_000001_create_attestation_type;
mod m20250925_000002_create_attestation_batch;
mod m20250925_000003_create_status_lists;
mod m20250925_000004_create_status_list_item;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250925_000001_create_attestation_type::Migration),
            Box::new(m20250925_000002_create_attestation_batch::Migration),
            Box::new(m20250925_000003_create_status_lists::Migration),
            Box::new(m20250925_000004_create_status_list_item::Migration),
        ]
    }
}
