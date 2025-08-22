#import "frb_generated.h"
#import "frb_rust.h"

static int64_t dummy_method_to_enforce_bundling_frb(void) {
    int64_t dummy_var = 0;
    dummy_var ^= ((int64_t) (void*) frb_pde_ffi_dispatcher_primary);
    dummy_var ^= ((int64_t) (void*) frb_pde_ffi_dispatcher_sync);
    dummy_var ^= ((int64_t) (void*) frb_dart_fn_deliver_output);
    dummy_var ^= ((int64_t) (void*) frb_get_rust_content_hash);
    dummy_var ^= ((int64_t) (void*) frb_dart_opaque_dart2rust_encode);
    dummy_var ^= ((int64_t) (void*) frb_dart_opaque_drop_thread_box_persistent_handle);
    dummy_var ^= ((int64_t) (void*) frb_dart_opaque_rust2dart_decode);
    dummy_var ^= ((int64_t) (void*) frb_rust_vec_u8_new);
    dummy_var ^= ((int64_t) (void*) frb_rust_vec_u8_resize);
    dummy_var ^= ((int64_t) (void*) frb_rust_vec_u8_free);
    dummy_var ^= ((int64_t) (void*) frb_init_frb_dart_api_dl);
    dummy_var ^= ((int64_t) (void*) frb_free_wire_sync_rust2dart_dco);
    dummy_var ^= ((int64_t) (void*) frb_free_wire_sync_rust2dart_sse);
    dummy_var ^= ((int64_t) (void*) frb_create_shutdown_callback);
    return dummy_var;
}
