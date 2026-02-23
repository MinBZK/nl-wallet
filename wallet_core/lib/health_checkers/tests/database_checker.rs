use rstest::rstest;
use sea_orm::Database;

use db_test::DbSetup;
use db_test::default_connection_options;
use health_checkers::postgres::DatabaseChecker;
use http_utils::health::HealthChecker;
use http_utils::health::HealthStatus;

#[tokio::test(flavor = "multi_thread")]
async fn test_db_check_up() {
    let db_setup = DbSetup::create().await;
    let connection = Database::connect(db_setup.connect_url()).await.unwrap();

    // Check
    let checker = DatabaseChecker::new("db", &connection);
    let result = checker.status().await;
    assert_eq!(result.expect("checker should return ok"), HealthStatus::UP);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[rstest]
async fn test_db_check_down(#[values(false, true)] test_before_acquire: bool) {
    let db_setup = DbSetup::create().await;
    let mut url = db_setup.connect_url();

    // Create incorrect connection (with lazy connect)
    url.set_password(Some("incorrect")).unwrap();
    let mut connect_options = default_connection_options(url);
    connect_options
        .connect_lazy(true)
        .test_before_acquire(test_before_acquire);
    let connection = Database::connect(connect_options).await.unwrap();

    // Check
    let checker = DatabaseChecker::new("db", &connection);
    let result = checker.status().await;
    assert!(result.is_err());
}
