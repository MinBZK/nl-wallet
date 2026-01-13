use tokio::runtime;

use crate::attested_key::hardware::HardwareAttestedKeyHolder;
use crate::attested_key::test::TestData;
use crate::attested_key::test::create_and_verify_attested_key;

fn attested_key_test(test_data: TestData) {
    let challenge = b"this_is_a_challenge_string";
    let payload = b"This is a message that will be signed.";

    super::print_panic(|| {
        let holder = HardwareAttestedKeyHolder::default();
        let rt = runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(create_and_verify_attested_key(
            &holder,
            test_data,
            challenge.to_vec(),
            payload.to_vec(),
        ));
    })
}

#[cfg(target_os = "android")]
mod android {
    use jni::JNIEnv;
    use jni::objects::JClass;
    use log::LevelFilter;

    use android_attest::root_public_key::GOOGLE_ROOT_PUBKEYS;
    use android_logger::Config;

    use crate::attested_key::test::AndroidTestData;
    use crate::attested_key::test::TestData;

    #[unsafe(no_mangle)]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_attested_1key_AttestedKeyBridgeInstrumentedTest_attested_1key_1test(
        _env: JNIEnv,
        _: JClass,
    ) {
        android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
        log::info!("Begin attested key test");

        let test_data = TestData::Android(AndroidTestData {
            root_public_keys: GOOGLE_ROOT_PUBKEYS.clone(),
        });
        super::attested_key_test(test_data);
    }
}

#[cfg(target_os = "ios")]
mod ios {
    use apple_app_attest::APPLE_TRUST_ANCHORS;
    use apple_app_attest::AppIdentifier;

    use crate::attested_key::test::AppleTestData;
    use crate::attested_key::test::TestData;

    #[unsafe(no_mangle)]
    extern "C" fn ios_attested_key_test() {
        let test_data = TestData::Apple(AppleTestData {
            app_identifier: &AppIdentifier::default(),
            trust_anchors: APPLE_TRUST_ANCHORS.clone(),
        });
        super::attested_key_test(test_data);
    }
}
