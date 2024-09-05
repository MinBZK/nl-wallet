use jni::{objects::JClass, JNIEnv};

use crate::utils::{hardware::HardwareUtilities, test};

// this is the starting point for the integration test performed from Android / iOS.
#[no_mangle]
fn utils_test_get_storage_path() -> bool {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(test::get_and_verify_storage_path::<HardwareUtilities>())
}

#[rustfmt::skip]
#[no_mangle]
extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_utilities_UtilitiesBridgeInstrumentedTest_utilities_1test_1storage_1path(
    _env: JNIEnv,
    _: JClass,
) -> bool {
    utils_test_get_storage_path()
}
