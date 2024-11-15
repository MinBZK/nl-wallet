use tokio::runtime;

use crate::attested_key::{hardware::HardwareAttestedKeyHolder, test};

#[no_mangle]
extern "C" fn attested_key_test() {
    let challenge = b"this_is_a_challenge_string";
    let payload = b"This is a message that will be signed.";

    super::print_panic(|| {
        let rt = runtime::Builder::new_current_thread().enable_all().build().unwrap();

        let holder = HardwareAttestedKeyHolder::default();

        rt.block_on(test::create_and_verify_attested_key(
            holder,
            challenge.to_vec(),
            payload.to_vec(),
        ));
    });
}

#[cfg(target_os = "android")]
mod android {
    use jni::{objects::JClass, JNIEnv};

    #[rustfmt::skip]
    #[no_mangle]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_attested_1key_AttestedKeyBridgeInstrumentedTest_attested_1key_1test(
        _env: JNIEnv,
        _: JClass,
    ) {
        super::attested_key_test();
    }
}
