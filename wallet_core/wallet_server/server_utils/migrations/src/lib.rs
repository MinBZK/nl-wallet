use async_trait::async_trait;

pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_session_table;
mod m20241023_095134_create_wua_table;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_session_table::Migration),
            Box::new(m20241023_095134_create_wua_table::Migration),
        ]
    }
}
