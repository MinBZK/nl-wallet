use crate::close_proximity_disclosure::hardware::HardwareCloseProximityDisclosureClient;
use crate::close_proximity_disclosure::test;

// this is the starting point for the integration test performed from Android / iOS.
#[unsafe(no_mangle)]
extern "C" fn close_proximity_disclosure_test_start_qr_handover() {
    super::print_panic(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(test::test_start_qr_handover::<HardwareCloseProximityDisclosureClient>())
    })
}

#[cfg(target_os = "android")]
mod android {
    use jni::JNIEnv;
    use jni::objects::JClass;

    #[unsafe(no_mangle)]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_close_1proximity_1disclosure_CloseProximityDisclosureBridgeInstrumentedTest_close_1proximity_1disclosure_1test_1start_1qr_1handover(
        _env: JNIEnv,
        _: JClass,
    ) {
        super::close_proximity_disclosure_test_start_qr_handover()
    }
}
