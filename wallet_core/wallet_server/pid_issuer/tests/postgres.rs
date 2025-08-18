use std::sync::Arc;

use openid4vc::server_state::test;
use pid_issuer::settings::PidIssuerSettings;
use pid_issuer::wua_tracker::PostgresWuaTracker;
use server_utils::settings::ServerSettings;
use server_utils::settings::Storage;
use server_utils::store::postgres;
use utils::generator::mock::MockTimeGenerator;

fn storage_settings() -> Storage {
    PidIssuerSettings::new("pid_issuer.toml", "pid_issuer")
        .unwrap()
        .issuer_settings
        .server_settings
        .storage
}

#[tokio::test]
async fn test_wua_tracker() {
    let time_generator = MockTimeGenerator::default();
    let mock_time = Arc::clone(&time_generator.time);

    let wua_tracker = PostgresWuaTracker::new_with_time(
        postgres::new_connection(storage_settings().url).await.unwrap(),
        time_generator,
    );

    test::test_wua_tracker(&wua_tracker, mock_time.as_ref()).await;
}
