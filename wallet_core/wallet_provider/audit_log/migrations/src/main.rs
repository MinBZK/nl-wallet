use sea_orm_migration::prelude::*;

mod m20260113_000001_create_audit_log;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260113_000001_create_audit_log::Migration)]
    }
}

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
