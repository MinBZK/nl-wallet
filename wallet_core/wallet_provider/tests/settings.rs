use wallet_provider::settings::Settings;

#[test]
fn test_load_settings() {
    Settings::new().expect("should load settings");
}
