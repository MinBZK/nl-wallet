use sea_orm_migration::prelude::*;

use wallet_server_migration::Migrator;

#[async_std::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
