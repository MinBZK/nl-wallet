use std::error::Error;

use sea_orm_migration::prelude::*;

use wallet_provider_database_settings::Settings;

mod m20230616_000001_create_wallet_user_table;
mod m20230908_000001_create_wallet_user_key_table;
mod m20230926_000001_create_wallet_user_challenge_instruction;
mod m20241118_104300_create_wallet_user_apple_attestation_table;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230616_000001_create_wallet_user_table::Migration),
            Box::new(m20230908_000001_create_wallet_user_key_table::Migration),
            Box::new(m20230926_000001_create_wallet_user_challenge_instruction::Migration),
            Box::new(m20241118_104300_create_wallet_user_apple_attestation_table::Migration),
        ]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new()?;

    std::env::set_var("DATABASE_URL", settings.database.connection_string());
    cli::run_cli(Migrator).await;

    Ok(())
}
