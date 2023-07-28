use wallet_provider::settings::Settings;

#[test]
#[cfg_attr(not(feature = "db_test"), ignore)]
fn test_load_settings() {
    Settings::new().expect("should load settings");
}
