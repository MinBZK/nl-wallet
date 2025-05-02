use pid_issuer::settings::PidIssuerSettings;
use server_utils::settings::ServerSettings;

#[test]
fn test_settings_success() {
    let settings = PidIssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    settings.validate().expect("should succeed");
}
