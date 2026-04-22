use audit_log_migrations::Migrator;
use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
