use crate::iso18013_5::hardware::HardwareIso18013_5SessionManager;
use crate::iso18013_5::test;

// this is the starting point for the integration test performed from Android / iOS.
#[unsafe(no_mangle)]
extern "C" fn iso18013_5_test_start_qr_handover() {
    super::print_panic(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(test::test_start_qr_handover::<HardwareIso18013_5SessionManager>())
    })
}

#[cfg(target_os = "android")]
mod android {
    use jni::JNIEnv;
    use jni::objects::JClass;

    #[unsafe(no_mangle)]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_iso180135_Iso180135BridgeInstrumentedTest_iso18013_15_1test_1start_1qr_1handover(
        _env: JNIEnv,
        _: JClass,
    ) {
        super::iso18013_5_test_start_qr_handover()
    }
}
