use std::error::Error;

use sea_orm_migration::prelude::*;

use wallet_provider_database_settings::Settings;

mod m20250102_000000_create_wallet_user_apple_attestation_table;
mod m20250102_000001_create_wallet_user_android_attestation_table;
mod m20250102_000010_create_wallet_user_table;
mod m20250102_000020_create_wallet_user_key_table;
mod m20250102_000021_create_wallet_user_challenge_instruction;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250102_000000_create_wallet_user_apple_attestation_table::Migration),
            Box::new(m20250102_000001_create_wallet_user_android_attestation_table::Migration),
            Box::new(m20250102_000010_create_wallet_user_table::Migration),
            Box::new(m20250102_000020_create_wallet_user_key_table::Migration),
            Box::new(m20250102_000021_create_wallet_user_challenge_instruction::Migration),
        ]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new()?;

    unsafe {
        std::env::set_var("DATABASE_URL", settings.database.connection_string());
    }
    cli::run_cli(Migrator).await;

    Ok(())
}
