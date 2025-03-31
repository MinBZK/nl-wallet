use std::sync::LazyLock;

use tokio::runtime;

use android_attest::root_public_key::EMULATOR_PUBKEYS;
use android_attest::root_public_key::GOOGLE_ROOT_PUBKEYS;
use apple_app_attest::AppIdentifier;
use apple_app_attest::APPLE_TRUST_ANCHORS;

use crate::attested_key::hardware::HardwareAttestedKeyHolder;
use crate::attested_key::test;
use crate::attested_key::test::AndroidTestData;
use crate::attested_key::test::AppleTestData;
use crate::attested_key::test::TestData;

#[no_mangle]
extern "C" fn attested_key_test(has_xcode_env: bool) {
    let challenge = b"this_is_a_challenge_string";
    let payload = b"This is a message that will be signed.";

    super::print_panic(|| {
        // Prepare Apple test data if we are executed from Xcode.
        let app_identifier = has_xcode_env.then(AppIdentifier::default);
        let test_data = app_identifier
            .as_ref()
            .map(|app_identifier| {
                TestData::Apple(AppleTestData {
                    app_identifier,
                    trust_anchors: APPLE_TRUST_ANCHORS.clone(),
                })
            })
            .unwrap_or(TestData::Android(AndroidTestData {
                root_public_keys: LazyLock::force(&GOOGLE_ROOT_PUBKEYS)
                    .iter()
                    .chain(LazyLock::force(&EMULATOR_PUBKEYS))
                    .cloned()
                    .collect(),
            }));

        let rt = runtime::Builder::new_current_thread().enable_all().build().unwrap();

        let holder = HardwareAttestedKeyHolder::default();

        rt.block_on(test::create_and_verify_attested_key(
            &holder,
            test_data,
            challenge.to_vec(),
            payload.to_vec(),
        ));
    });
}

#[cfg(target_os = "android")]
mod android {
    use android_logger::Config;
    use jni::objects::JClass;
    use jni::JNIEnv;
    use log::LevelFilter;

    #[rustfmt::skip]
    #[no_mangle]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_attested_1key_AttestedKeyBridgeInstrumentedTest_attested_1key_1test(
        _env: JNIEnv,
        _: JClass,
    ) {
        android_logger::init_once(
            Config::default().with_max_level(LevelFilter::Trace),
        );
        log::info!("Begin attested key test");
        super::attested_key_test(false);
    }
}
