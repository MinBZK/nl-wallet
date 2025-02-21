use std::sync::Arc;

use pid_issuer::settings::IssuerSettings;

use openid4vc::server_state::test;
use openid4vc_server::store::postgres;
use openid4vc_server::store::postgres::PostgresWteTracker;
use wallet_common::generator::mock::MockTimeGenerator;
use wallet_server::settings::ServerSettings;
use wallet_server::settings::Storage;

fn storage_settings() -> Storage {
    IssuerSettings::new_custom("pid_issuer.toml", "pid_issuer")
        .unwrap()
        .server_settings
        .storage
}

#[tokio::test]
async fn test_wte_tracker() {
    let time_generator = MockTimeGenerator::default();
    let mock_time = Arc::clone(&time_generator.time);

    let wte_tracker = PostgresWteTracker::new_with_time(
        postgres::new_connection(storage_settings().url).await.unwrap(),
        time_generator,
    );

    test::test_wte_tracker(&wte_tracker, mock_time.as_ref()).await;
}
