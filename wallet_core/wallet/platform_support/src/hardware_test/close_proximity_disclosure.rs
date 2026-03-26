use crate::close_proximity_disclosure::CloseProximityDisclosureClient;
use crate::close_proximity_disclosure::hardware::HardwareCloseProximityDisclosureClient;

// this is the starting point for the integration test performed from Android / iOS.
#[unsafe(no_mangle)]
extern "C" fn close_proximity_disclosure_test_start_qr_handover() {
    super::print_panic(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let (_qr, _receiver) = HardwareCloseProximityDisclosureClient::start_qr_handover()
                .await
                .unwrap();
            HardwareCloseProximityDisclosureClient::stop_ble_server().await.unwrap();
        })
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
