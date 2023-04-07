use super::get_and_verify_storage_path;
use crate::utils::hardware::HardwareUtilities;
use jni::{objects::JClass, JNIEnv};

// this is the starting point for integration test performed from Android / iOS.
#[no_mangle]
fn utils_test_get_storage_path() -> bool {
    get_and_verify_storage_path::<HardwareUtilities>()
}

#[no_mangle]
extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_utilities_NativeUtilitiesBridgeInstrumentedTest_utilities_1test_1storage_1path(
    _env: JNIEnv,
    _: JClass,
) -> bool {
    utils_test_get_storage_path()
}
