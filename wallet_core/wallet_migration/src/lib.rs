pub use sea_orm_migration::prelude::*;

mod m20230425_140221_create_keyed_data_table;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20230425_140221_create_keyed_data_table::Migration)]
    }
}
