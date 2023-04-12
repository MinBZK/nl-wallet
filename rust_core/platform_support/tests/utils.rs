#[cfg(feature = "software-integration-test")]
#[test]
fn test_storage_path() {
    use platform_support::utils::{integration_test::get_and_verify_storage_path, software::SoftwareUtilities};

    assert!(get_and_verify_storage_path::<SoftwareUtilities>());
}
