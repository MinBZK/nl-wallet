use crate::utils::hardware::HardwareUtilities;
use crate::utils::test;

// this is the starting point for the integration test performed from Android / iOS.
#[unsafe(no_mangle)]
extern "C" fn utils_test_get_storage_path() {
    super::print_panic(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(test::get_and_verify_storage_path::<HardwareUtilities>())
    })
}

#[cfg(target_os = "android")]
mod android {
    use jni::JNIEnv;
    use jni::objects::JClass;

    #[rustfmt::skip]
    #[unsafe(no_mangle)]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_utilities_UtilitiesBridgeInstrumentedTest_utilities_1test_1storage_1path(
        _env: JNIEnv,
        _: JClass,
    ) {
        super::utils_test_get_storage_path()
    }
}
