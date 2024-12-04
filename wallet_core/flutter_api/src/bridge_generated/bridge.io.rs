use super::*;
// Section: wire functions

#[no_mangle]
pub extern "C" fn wire_init(port_: i64) {
    wire_init_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_is_initialized(port_: i64) {
    wire_is_initialized_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_is_valid_pin(port_: i64, pin: *mut wire_uint_8_list) {
    wire_is_valid_pin_impl(port_, pin)
}

#[no_mangle]
pub extern "C" fn wire_set_lock_stream(port_: i64) {
    wire_set_lock_stream_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_clear_lock_stream(port_: i64) {
    wire_clear_lock_stream_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_set_configuration_stream(port_: i64) {
    wire_set_configuration_stream_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_clear_configuration_stream(port_: i64) {
    wire_clear_configuration_stream_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_set_cards_stream(port_: i64) {
    wire_set_cards_stream_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_clear_cards_stream(port_: i64) {
    wire_clear_cards_stream_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_set_recent_history_stream(port_: i64) {
    wire_set_recent_history_stream_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_clear_recent_history_stream(port_: i64) {
    wire_clear_recent_history_stream_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_unlock_wallet(port_: i64, pin: *mut wire_uint_8_list) {
    wire_unlock_wallet_impl(port_, pin)
}

#[no_mangle]
pub extern "C" fn wire_lock_wallet(port_: i64) {
    wire_lock_wallet_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_check_pin(port_: i64, pin: *mut wire_uint_8_list) {
    wire_check_pin_impl(port_, pin)
}

#[no_mangle]
pub extern "C" fn wire_change_pin(port_: i64, old_pin: *mut wire_uint_8_list, new_pin: *mut wire_uint_8_list) {
    wire_change_pin_impl(port_, old_pin, new_pin)
}

#[no_mangle]
pub extern "C" fn wire_continue_change_pin(port_: i64, pin: *mut wire_uint_8_list) {
    wire_continue_change_pin_impl(port_, pin)
}

#[no_mangle]
pub extern "C" fn wire_has_registration(port_: i64) {
    wire_has_registration_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_register(port_: i64, pin: *mut wire_uint_8_list) {
    wire_register_impl(port_, pin)
}

#[no_mangle]
pub extern "C" fn wire_identify_uri(port_: i64, uri: *mut wire_uint_8_list) {
    wire_identify_uri_impl(port_, uri)
}

#[no_mangle]
pub extern "C" fn wire_create_pid_issuance_redirect_uri(port_: i64) {
    wire_create_pid_issuance_redirect_uri_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_cancel_pid_issuance(port_: i64) {
    wire_cancel_pid_issuance_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_continue_pid_issuance(port_: i64, uri: *mut wire_uint_8_list) {
    wire_continue_pid_issuance_impl(port_, uri)
}

#[no_mangle]
pub extern "C" fn wire_accept_pid_issuance(port_: i64, pin: *mut wire_uint_8_list) {
    wire_accept_pid_issuance_impl(port_, pin)
}

#[no_mangle]
pub extern "C" fn wire_has_active_pid_issuance_session(port_: i64) {
    wire_has_active_pid_issuance_session_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_start_disclosure(port_: i64, uri: *mut wire_uint_8_list, is_qr_code: bool) {
    wire_start_disclosure_impl(port_, uri, is_qr_code)
}

#[no_mangle]
pub extern "C" fn wire_cancel_disclosure(port_: i64) {
    wire_cancel_disclosure_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_accept_disclosure(port_: i64, pin: *mut wire_uint_8_list) {
    wire_accept_disclosure_impl(port_, pin)
}

#[no_mangle]
pub extern "C" fn wire_has_active_disclosure_session(port_: i64) {
    wire_has_active_disclosure_session_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_is_biometric_unlock_enabled(port_: i64) {
    wire_is_biometric_unlock_enabled_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_set_biometric_unlock(port_: i64, enable: bool) {
    wire_set_biometric_unlock_impl(port_, enable)
}

#[no_mangle]
pub extern "C" fn wire_unlock_wallet_with_biometrics(port_: i64) {
    wire_unlock_wallet_with_biometrics_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_get_history(port_: i64) {
    wire_get_history_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_get_history_for_card(port_: i64, doc_type: *mut wire_uint_8_list) {
    wire_get_history_for_card_impl(port_, doc_type)
}

#[no_mangle]
pub extern "C" fn wire_reset_wallet(port_: i64) {
    wire_reset_wallet_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_get_version_string(port_: i64) {
    wire_get_version_string_impl(port_)
}

// Section: allocate functions

#[no_mangle]
pub extern "C" fn new_uint_8_list_0(len: i32) -> *mut wire_uint_8_list {
    let ans = wire_uint_8_list {
        ptr: support::new_leak_vec_ptr(Default::default(), len),
        len,
    };
    support::new_leak_box_ptr(ans)
}

// Section: related functions

// Section: impl Wire2Api

impl Wire2Api<String> for *mut wire_uint_8_list {
    fn wire2api(self) -> String {
        let vec: Vec<u8> = self.wire2api();
        String::from_utf8_lossy(&vec).into_owned()
    }
}

impl Wire2Api<Vec<u8>> for *mut wire_uint_8_list {
    fn wire2api(self) -> Vec<u8> {
        unsafe {
            let wrap = support::box_from_leak_ptr(self);
            support::vec_from_leak_ptr(wrap.ptr, wrap.len)
        }
    }
}
// Section: wire structs

#[repr(C)]
#[derive(Clone)]
pub struct wire_uint_8_list {
    ptr: *mut u8,
    len: i32,
}

// Section: impl NewWithNullPtr

pub trait NewWithNullPtr {
    fn new_with_null_ptr() -> Self;
}

impl<T> NewWithNullPtr for *mut T {
    fn new_with_null_ptr() -> Self {
        std::ptr::null_mut()
    }
}

// Section: sync execution mode utility

#[no_mangle]
pub extern "C" fn free_WireSyncReturn(ptr: support::WireSyncReturn) {
    unsafe {
        let _ = support::box_from_leak_ptr(ptr);
    };
}
