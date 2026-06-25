use acf_demo_issuer::settings::AcfDemoIssuerSettings;
use server_utils::settings::ServerSettings;

#[test]
fn test_settings_success() {
    let settings = AcfDemoIssuerSettings::new("acf_demo_issuer.toml", "acf_demo_issuer").expect("default settings");

    settings.validate().expect("should succeed");
}
