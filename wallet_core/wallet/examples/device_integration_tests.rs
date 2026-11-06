use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::time::Duration;

use anyhow::Context;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureClient;
use platform_support::close_proximity_disclosure::hardware::HardwareCloseProximityDisclosureClient;

#[derive(Clone, Copy)]
enum DeviceTest<'a> {
    CwtDisclosure,
    JwtDisclosure,
    Rejection(&'a CStr),
}

/// Run off the iOS main thread. The callback receives a borrowed string, valid only during the call.
/// Return null on success, otherwise an owned error to release with `wallet_device_test_free_error`.
#[unsafe(no_mangle)]
pub extern "C" fn wallet_close_proximity_disclosure_test(reader_ready: extern "C" fn(*const c_char)) -> *mut c_char {
    run_test(DeviceTest::CwtDisclosure, reader_ready)
}

/// Run the disclosure scenario using a JWT registration certificate. Ownership matches the CWT entry point.
#[unsafe(no_mangle)]
pub extern "C" fn wallet_jwt_registration_certificate_test(reader_ready: extern "C" fn(*const c_char)) -> *mut c_char {
    run_test(DeviceTest::JwtDisclosure, reader_ready)
}

/// Run a named EUDI registration-certificate rejection case.
///
/// # Safety
/// `case` must point to a NUL-terminated UTF-8 string, valid for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wallet_registration_certificate_test(
    case: *const c_char,
    reader_ready: extern "C" fn(*const c_char),
) -> *mut c_char {
    // SAFETY: the caller keeps the case string alive until this function returns.
    let case = unsafe { CStr::from_ptr(case) };
    run_test(DeviceTest::Rejection(case), reader_ready)
}

fn run_test(test: DeviceTest<'_>, reader_ready: extern "C" fn(*const c_char)) -> *mut c_char {
    let result = catch_unwind(|| -> anyhow::Result<()> {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        let result = catch_unwind(AssertUnwindSafe(|| {
            let notify_reader = |ready| {
                let marker = CString::new(format!(
                    "CLOSE_PROXIMITY_MAC_READER={}",
                    serde_json::to_string(&ready).unwrap()
                ))
                .expect("JSON has no interior NUL bytes");
                reader_ready(marker.as_ptr());
            };
            runtime.block_on(async {
                match test {
                    DeviceTest::Rejection(case) => {
                        let case = case
                            .to_str()?
                            .parse()
                            .context("unknown registration certificate case")?;
                        wallet::hardware_test::run_registration_certificate_rejection::<
                            HardwareCloseProximityDisclosureClient,
                        >(case, notify_reader)
                        .await
                    }
                    DeviceTest::CwtDisclosure => {
                        wallet::hardware_test::run_disclosure::<HardwareCloseProximityDisclosureClient>(notify_reader)
                            .await
                    }
                    DeviceTest::JwtDisclosure => {
                        wallet::hardware_test::run_disclosure_with_jwt::<HardwareCloseProximityDisclosureClient>(
                            notify_reader,
                        )
                        .await
                    }
                }
            })
        }));
        match result {
            Ok(result) => result,
            Err(_) => {
                // Normal failures clean up in the scenario; a panicking fixture must also release BLE.
                let _ = runtime.block_on(async {
                    tokio::time::timeout(
                        Duration::from_secs(5),
                        HardwareCloseProximityDisclosureClient::stop_ble_server(),
                    )
                    .await
                });
                Err(anyhow::anyhow!("Rust device test panicked")).context("see the test process log")
            }
        }
    });
    let message = match result {
        Ok(Ok(())) => return std::ptr::null_mut(),
        Ok(Err(error)) => format!("{error:#}"),
        Err(_) => "Rust device test panicked; see the test process log".to_owned(),
    };
    CString::new(message.replace('\0', "\\0"))
        .expect("NUL bytes replaced")
        .into_raw()
}

/// # Safety
/// `error` must be a non-null pointer returned by a wallet device test, freed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wallet_device_test_free_error(error: *mut c_char) {
    // SAFETY: the caller transfers ownership of the error string back to Rust.
    drop(unsafe { CString::from_raw(error) });
}
