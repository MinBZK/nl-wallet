use acf_demo_issuer_migrations::Migrator;

#[tokio::main]
async fn main() {
    sea_orm_migration::cli::run_cli(Migrator).await;
}
