use sea_orm_migration::prelude::*;

mod m20250102_000000_create_wallet_user_apple_attestation_table;
mod m20250102_000001_create_wallet_user_android_attestation_table;
mod m20250102_000010_create_wallet_user_table;
mod m20250102_000020_create_wallet_user_key_table;
mod m20250102_000021_create_wallet_user_challenge_instruction;
mod m20250102_000030_create_wallet_transfer_table;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250102_000000_create_wallet_user_apple_attestation_table::Migration),
            Box::new(m20250102_000001_create_wallet_user_android_attestation_table::Migration),
            Box::new(m20250102_000010_create_wallet_user_table::Migration),
            Box::new(m20250102_000020_create_wallet_user_key_table::Migration),
            Box::new(m20250102_000021_create_wallet_user_challenge_instruction::Migration),
            Box::new(m20250102_000030_create_wallet_transfer_table::Migration),
        ]
    }
}

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
