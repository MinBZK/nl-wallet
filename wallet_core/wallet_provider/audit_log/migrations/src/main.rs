use sea_orm_migration::prelude::*;

use audit_log_migrations::Migrator;

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
