use sea_orm_migration::prelude::*;

mod m20241023_095134_create_wua_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = server_utils_migrations::Migrator::migrations();
        migrations.push(Box::new(m20241023_095134_create_wua_table::Migration));
        migrations
    }
}

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
