use std::error::Error;

use sea_orm_migration::prelude::*;

use wallet_provider::settings::Settings;
use wallet_provider_migration::Migrator;
use wallet_provider_persistence::postgres::connection_string;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new().unwrap();

    let url = connection_string(
        &settings.database.host,
        &settings.database.name,
        settings.database.username.as_deref(),
        settings.database.password.as_deref(),
    );

    std::env::set_var("DATABASE_URL", url);
    cli::run_cli(Migrator).await;

    Ok(())
}
