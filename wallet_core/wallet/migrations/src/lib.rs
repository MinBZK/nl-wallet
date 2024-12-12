pub use sea_orm_migration::prelude::*;

mod m20230425_140221_create_keyed_data_table;
mod m20230922_095234_create_mdoc_tables;
mod m20231115_100948_create_history_tables;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230425_140221_create_keyed_data_table::Migration),
            Box::new(m20230922_095234_create_mdoc_tables::Migration),
            Box::new(m20231115_100948_create_history_tables::Migration),
        ]
    }
}
