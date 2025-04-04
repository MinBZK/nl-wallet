use issuance_server::settings::IssuanceServerSettings;
use server_utils::settings::ServerSettings;

#[test]
fn test_settings_success() {
    let settings = IssuanceServerSettings::new("issuance_server.toml", "issuance_server").expect("default settings");

    settings.validate().expect("should succeed");
}
