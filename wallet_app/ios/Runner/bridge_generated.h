#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
typedef struct _Dart_Handle* Dart_Handle;

typedef struct DartCObject DartCObject;

typedef int64_t DartPort;

typedef bool (*DartPostCObjectFnType)(DartPort port_id, void *message);

typedef struct wire_uint_8_list {
  uint8_t *ptr;
  int32_t len;
} wire_uint_8_list;

typedef struct DartCObject *WireSyncReturn;

void store_dart_post_cobject(DartPostCObjectFnType ptr);

Dart_Handle get_dart_object(uintptr_t ptr);

void drop_dart_object(uintptr_t ptr);

uintptr_t new_dart_opaque(Dart_Handle handle);

intptr_t init_frb_dart_api_dl(void *obj);

void wire_init(int64_t port_);

void wire_is_initialized(int64_t port_);

void wire_is_valid_pin(int64_t port_, struct wire_uint_8_list *pin);

void wire_set_lock_stream(int64_t port_);

void wire_clear_lock_stream(int64_t port_);

void wire_set_configuration_stream(int64_t port_);

void wire_clear_configuration_stream(int64_t port_);

void wire_set_cards_stream(int64_t port_);

void wire_clear_cards_stream(int64_t port_);

void wire_set_recent_history_stream(int64_t port_);

void wire_clear_recent_history_stream(int64_t port_);

void wire_unlock_wallet(int64_t port_, struct wire_uint_8_list *pin);

void wire_lock_wallet(int64_t port_);

void wire_check_pin(int64_t port_, struct wire_uint_8_list *pin);

void wire_change_pin(int64_t port_,
                     struct wire_uint_8_list *old_pin,
                     struct wire_uint_8_list *new_pin);

void wire_continue_change_pin(int64_t port_, struct wire_uint_8_list *pin);

void wire_has_registration(int64_t port_);

void wire_register(int64_t port_, struct wire_uint_8_list *pin);

void wire_identify_uri(int64_t port_, struct wire_uint_8_list *uri);

void wire_create_pid_issuance_redirect_uri(int64_t port_);

void wire_cancel_pid_issuance(int64_t port_);

void wire_continue_pid_issuance(int64_t port_, struct wire_uint_8_list *uri);

void wire_accept_pid_issuance(int64_t port_, struct wire_uint_8_list *pin);

void wire_has_active_pid_issuance_session(int64_t port_);

void wire_start_disclosure(int64_t port_, struct wire_uint_8_list *uri, bool is_qr_code);

void wire_cancel_disclosure(int64_t port_);

void wire_accept_disclosure(int64_t port_, struct wire_uint_8_list *pin);

void wire_has_active_disclosure_session(int64_t port_);

void wire_is_biometric_unlock_enabled(int64_t port_);

void wire_set_biometric_unlock(int64_t port_, bool enable);

void wire_unlock_wallet_with_biometrics(int64_t port_);

void wire_get_history(int64_t port_);

void wire_get_history_for_card(int64_t port_, struct wire_uint_8_list *doc_type);

void wire_reset_wallet(int64_t port_);

struct wire_uint_8_list *new_uint_8_list_0(int32_t len);

void free_WireSyncReturn(WireSyncReturn ptr);

static int64_t dummy_method_to_enforce_bundling(void) {
    int64_t dummy_var = 0;
    dummy_var ^= ((int64_t) (void*) wire_init);
    dummy_var ^= ((int64_t) (void*) wire_is_initialized);
    dummy_var ^= ((int64_t) (void*) wire_is_valid_pin);
    dummy_var ^= ((int64_t) (void*) wire_set_lock_stream);
    dummy_var ^= ((int64_t) (void*) wire_clear_lock_stream);
    dummy_var ^= ((int64_t) (void*) wire_set_configuration_stream);
    dummy_var ^= ((int64_t) (void*) wire_clear_configuration_stream);
    dummy_var ^= ((int64_t) (void*) wire_set_cards_stream);
    dummy_var ^= ((int64_t) (void*) wire_clear_cards_stream);
    dummy_var ^= ((int64_t) (void*) wire_set_recent_history_stream);
    dummy_var ^= ((int64_t) (void*) wire_clear_recent_history_stream);
    dummy_var ^= ((int64_t) (void*) wire_unlock_wallet);
    dummy_var ^= ((int64_t) (void*) wire_lock_wallet);
    dummy_var ^= ((int64_t) (void*) wire_check_pin);
    dummy_var ^= ((int64_t) (void*) wire_change_pin);
    dummy_var ^= ((int64_t) (void*) wire_continue_change_pin);
    dummy_var ^= ((int64_t) (void*) wire_has_registration);
    dummy_var ^= ((int64_t) (void*) wire_register);
    dummy_var ^= ((int64_t) (void*) wire_identify_uri);
    dummy_var ^= ((int64_t) (void*) wire_create_pid_issuance_redirect_uri);
    dummy_var ^= ((int64_t) (void*) wire_cancel_pid_issuance);
    dummy_var ^= ((int64_t) (void*) wire_continue_pid_issuance);
    dummy_var ^= ((int64_t) (void*) wire_accept_pid_issuance);
    dummy_var ^= ((int64_t) (void*) wire_has_active_pid_issuance_session);
    dummy_var ^= ((int64_t) (void*) wire_start_disclosure);
    dummy_var ^= ((int64_t) (void*) wire_cancel_disclosure);
    dummy_var ^= ((int64_t) (void*) wire_accept_disclosure);
    dummy_var ^= ((int64_t) (void*) wire_has_active_disclosure_session);
    dummy_var ^= ((int64_t) (void*) wire_is_biometric_unlock_enabled);
    dummy_var ^= ((int64_t) (void*) wire_set_biometric_unlock);
    dummy_var ^= ((int64_t) (void*) wire_unlock_wallet_with_biometrics);
    dummy_var ^= ((int64_t) (void*) wire_get_history);
    dummy_var ^= ((int64_t) (void*) wire_get_history_for_card);
    dummy_var ^= ((int64_t) (void*) wire_reset_wallet);
    dummy_var ^= ((int64_t) (void*) new_uint_8_list_0);
    dummy_var ^= ((int64_t) (void*) free_WireSyncReturn);
    dummy_var ^= ((int64_t) (void*) store_dart_post_cobject);
    dummy_var ^= ((int64_t) (void*) get_dart_object);
    dummy_var ^= ((int64_t) (void*) drop_dart_object);
    dummy_var ^= ((int64_t) (void*) new_dart_opaque);
    return dummy_var;
}
