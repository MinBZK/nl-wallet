use sea_orm_migration::prelude::*;

use wallet_migrations::Migrator;

/// Can be used to run the migrations against a local database and inspect the changes. See `README.md` for more
/// information.
#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
