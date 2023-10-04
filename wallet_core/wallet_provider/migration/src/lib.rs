pub use sea_orm_migration::prelude::*;

mod m20230616_000001_create_wallet_user_table;
mod m20230908_000001_create_wallet_user_key_table;
mod m20230926_000001_create_wallet_user_challenge_instruction;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230616_000001_create_wallet_user_table::Migration),
            Box::new(m20230908_000001_create_wallet_user_key_table::Migration),
            Box::new(m20230926_000001_create_wallet_user_challenge_instruction::Migration),
        ]
    }
}
