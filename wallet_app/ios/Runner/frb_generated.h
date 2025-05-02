#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
// EXTRA BEGIN
typedef struct DartCObject *WireSyncRust2DartDco;
typedef struct WireSyncRust2DartSse {
  uint8_t *ptr;
  int32_t len;
} WireSyncRust2DartSse;

typedef int64_t DartPort;
typedef bool (*DartPostCObjectFnType)(DartPort port_id, void *message);
void store_dart_post_cobject(DartPostCObjectFnType ptr);
// EXTRA END
typedef struct _Dart_Handle* Dart_Handle;

typedef struct wire_cst_list_prim_u_8_strict {
  uint8_t *ptr;
  int32_t len;
} wire_cst_list_prim_u_8_strict;

typedef struct wire_cst_AttestationIdentity_Fixed {
  struct wire_cst_list_prim_u_8_strict *id;
} wire_cst_AttestationIdentity_Fixed;

typedef union AttestationIdentityKind {
  struct wire_cst_AttestationIdentity_Fixed Fixed;
} AttestationIdentityKind;

typedef struct wire_cst_attestation_identity {
  int32_t tag;
  union AttestationIdentityKind kind;
} wire_cst_attestation_identity;

typedef struct wire_cst_Image_Svg {
  struct wire_cst_list_prim_u_8_strict *xml;
} wire_cst_Image_Svg;

typedef struct wire_cst_Image_Png {
  struct wire_cst_list_prim_u_8_strict *data;
} wire_cst_Image_Png;

typedef struct wire_cst_Image_Jpeg {
  struct wire_cst_list_prim_u_8_strict *data;
} wire_cst_Image_Jpeg;

typedef struct wire_cst_Image_Asset {
  struct wire_cst_list_prim_u_8_strict *path;
} wire_cst_Image_Asset;

typedef union ImageKind {
  struct wire_cst_Image_Svg Svg;
  struct wire_cst_Image_Png Png;
  struct wire_cst_Image_Jpeg Jpeg;
  struct wire_cst_Image_Asset Asset;
} ImageKind;

typedef struct wire_cst_image {
  int32_t tag;
  union ImageKind kind;
} wire_cst_image;

typedef struct wire_cst_image_with_metadata {
  struct wire_cst_image image;
  struct wire_cst_list_prim_u_8_strict *alt_text;
} wire_cst_image_with_metadata;

typedef struct wire_cst_RenderingMetadata_Simple {
  struct wire_cst_image_with_metadata *logo;
  struct wire_cst_list_prim_u_8_strict *background_color;
  struct wire_cst_list_prim_u_8_strict *text_color;
} wire_cst_RenderingMetadata_Simple;

typedef union RenderingMetadataKind {
  struct wire_cst_RenderingMetadata_Simple Simple;
} RenderingMetadataKind;

typedef struct wire_cst_rendering_metadata {
  int32_t tag;
  union RenderingMetadataKind kind;
} wire_cst_rendering_metadata;

typedef struct wire_cst_display_metadata {
  struct wire_cst_list_prim_u_8_strict *lang;
  struct wire_cst_list_prim_u_8_strict *name;
  struct wire_cst_list_prim_u_8_strict *description;
  struct wire_cst_list_prim_u_8_strict *summary;
  struct wire_cst_rendering_metadata *rendering;
} wire_cst_display_metadata;

typedef struct wire_cst_list_display_metadata {
  struct wire_cst_display_metadata *ptr;
  int32_t len;
} wire_cst_list_display_metadata;

typedef struct wire_cst_localized_string {
  struct wire_cst_list_prim_u_8_strict *language;
  struct wire_cst_list_prim_u_8_strict *value;
} wire_cst_localized_string;

typedef struct wire_cst_list_localized_string {
  struct wire_cst_localized_string *ptr;
  int32_t len;
} wire_cst_list_localized_string;

typedef struct wire_cst_organization {
  struct wire_cst_list_localized_string *legal_name;
  struct wire_cst_list_localized_string *display_name;
  struct wire_cst_list_localized_string *description;
  struct wire_cst_image *image;
  struct wire_cst_list_prim_u_8_strict *web_url;
  struct wire_cst_list_prim_u_8_strict *privacy_policy_url;
  struct wire_cst_list_prim_u_8_strict *kvk;
  struct wire_cst_list_localized_string *city;
  struct wire_cst_list_localized_string *category;
  struct wire_cst_list_localized_string *department;
  struct wire_cst_list_prim_u_8_strict *country_code;
} wire_cst_organization;

typedef struct wire_cst_claim_display_metadata {
  struct wire_cst_list_prim_u_8_strict *lang;
  struct wire_cst_list_prim_u_8_strict *label;
  struct wire_cst_list_prim_u_8_strict *description;
} wire_cst_claim_display_metadata;

typedef struct wire_cst_list_claim_display_metadata {
  struct wire_cst_claim_display_metadata *ptr;
  int32_t len;
} wire_cst_list_claim_display_metadata;

typedef struct wire_cst_AttributeValue_String {
  struct wire_cst_list_prim_u_8_strict *value;
} wire_cst_AttributeValue_String;

typedef struct wire_cst_AttributeValue_Boolean {
  bool value;
} wire_cst_AttributeValue_Boolean;

typedef struct wire_cst_AttributeValue_Number {
  int64_t value;
} wire_cst_AttributeValue_Number;

typedef struct wire_cst_AttributeValue_Date {
  struct wire_cst_list_prim_u_8_strict *value;
} wire_cst_AttributeValue_Date;

typedef union AttributeValueKind {
  struct wire_cst_AttributeValue_String String;
  struct wire_cst_AttributeValue_Boolean Boolean;
  struct wire_cst_AttributeValue_Number Number;
  struct wire_cst_AttributeValue_Date Date;
} AttributeValueKind;

typedef struct wire_cst_attribute_value {
  int32_t tag;
  union AttributeValueKind kind;
} wire_cst_attribute_value;

typedef struct wire_cst_attestation_attribute {
  struct wire_cst_list_prim_u_8_strict *key;
  struct wire_cst_list_claim_display_metadata *labels;
  struct wire_cst_attribute_value value;
  struct wire_cst_list_prim_u_8_strict *svg_id;
} wire_cst_attestation_attribute;

typedef struct wire_cst_list_attestation_attribute {
  struct wire_cst_attestation_attribute *ptr;
  int32_t len;
} wire_cst_list_attestation_attribute;

typedef struct wire_cst_attestation {
  struct wire_cst_attestation_identity identity;
  struct wire_cst_list_prim_u_8_strict *attestation_type;
  struct wire_cst_list_display_metadata *display_metadata;
  struct wire_cst_organization issuer;
  struct wire_cst_list_attestation_attribute *attributes;
} wire_cst_attestation;

typedef struct wire_cst_request_policy {
  uint64_t *data_storage_duration_in_minutes;
  bool data_shared_with_third_parties;
  bool data_deletion_possible;
  struct wire_cst_list_prim_u_8_strict *policy_url;
} wire_cst_request_policy;

typedef struct wire_cst_WalletInstructionError_IncorrectPin {
  uint8_t attempts_left_in_round;
  bool is_final_round;
} wire_cst_WalletInstructionError_IncorrectPin;

typedef struct wire_cst_WalletInstructionError_Timeout {
  uint64_t timeout_millis;
} wire_cst_WalletInstructionError_Timeout;

typedef union WalletInstructionErrorKind {
  struct wire_cst_WalletInstructionError_IncorrectPin IncorrectPin;
  struct wire_cst_WalletInstructionError_Timeout Timeout;
} WalletInstructionErrorKind;

typedef struct wire_cst_wallet_instruction_error {
  int32_t tag;
  union WalletInstructionErrorKind kind;
} wire_cst_wallet_instruction_error;

typedef struct wire_cst_list_attestation {
  struct wire_cst_attestation *ptr;
  int32_t len;
} wire_cst_list_attestation;

typedef struct wire_cst_missing_attribute {
  struct wire_cst_list_localized_string *labels;
} wire_cst_missing_attribute;

typedef struct wire_cst_list_missing_attribute {
  struct wire_cst_missing_attribute *ptr;
  int32_t len;
} wire_cst_list_missing_attribute;

typedef struct wire_cst_WalletEvent_Disclosure {
  struct wire_cst_list_prim_u_8_strict *date_time;
  struct wire_cst_organization *relying_party;
  struct wire_cst_list_localized_string *purpose;
  struct wire_cst_list_attestation *shared_attestations;
  struct wire_cst_request_policy *request_policy;
  int32_t status;
  int32_t typ;
} wire_cst_WalletEvent_Disclosure;

typedef struct wire_cst_WalletEvent_Issuance {
  struct wire_cst_list_prim_u_8_strict *date_time;
  struct wire_cst_attestation *attestation;
} wire_cst_WalletEvent_Issuance;

typedef union WalletEventKind {
  struct wire_cst_WalletEvent_Disclosure Disclosure;
  struct wire_cst_WalletEvent_Issuance Issuance;
} WalletEventKind;

typedef struct wire_cst_wallet_event {
  int32_t tag;
  union WalletEventKind kind;
} wire_cst_wallet_event;

typedef struct wire_cst_list_wallet_event {
  struct wire_cst_wallet_event *ptr;
  int32_t len;
} wire_cst_list_wallet_event;

typedef struct wire_cst_AcceptDisclosureResult_Ok {
  struct wire_cst_list_prim_u_8_strict *return_url;
} wire_cst_AcceptDisclosureResult_Ok;

typedef struct wire_cst_AcceptDisclosureResult_InstructionError {
  struct wire_cst_wallet_instruction_error *error;
} wire_cst_AcceptDisclosureResult_InstructionError;

typedef union AcceptDisclosureResultKind {
  struct wire_cst_AcceptDisclosureResult_Ok Ok;
  struct wire_cst_AcceptDisclosureResult_InstructionError InstructionError;
} AcceptDisclosureResultKind;

typedef struct wire_cst_accept_disclosure_result {
  int32_t tag;
  union AcceptDisclosureResultKind kind;
} wire_cst_accept_disclosure_result;

typedef struct wire_cst_flutter_configuration {
  uint16_t inactive_warning_timeout;
  uint16_t inactive_lock_timeout;
  uint16_t background_lock_timeout;
  uint64_t version;
} wire_cst_flutter_configuration;

typedef struct wire_cst_FlutterVersionState_Warn {
  uint64_t expires_in_seconds;
} wire_cst_FlutterVersionState_Warn;

typedef union FlutterVersionStateKind {
  struct wire_cst_FlutterVersionState_Warn Warn;
} FlutterVersionStateKind;

typedef struct wire_cst_flutter_version_state {
  int32_t tag;
  union FlutterVersionStateKind kind;
} wire_cst_flutter_version_state;

typedef struct wire_cst_StartDisclosureResult_Request {
  struct wire_cst_organization *relying_party;
  struct wire_cst_request_policy *policy;
  struct wire_cst_list_attestation *requested_attestations;
  bool shared_data_with_relying_party_before;
  int32_t session_type;
  struct wire_cst_list_localized_string *request_purpose;
  struct wire_cst_list_prim_u_8_strict *request_origin_base_url;
  int32_t request_type;
} wire_cst_StartDisclosureResult_Request;

typedef struct wire_cst_StartDisclosureResult_RequestAttributesMissing {
  struct wire_cst_organization *relying_party;
  struct wire_cst_list_missing_attribute *missing_attributes;
  bool shared_data_with_relying_party_before;
  int32_t session_type;
  struct wire_cst_list_localized_string *request_purpose;
  struct wire_cst_list_prim_u_8_strict *request_origin_base_url;
} wire_cst_StartDisclosureResult_RequestAttributesMissing;

typedef union StartDisclosureResultKind {
  struct wire_cst_StartDisclosureResult_Request Request;
  struct wire_cst_StartDisclosureResult_RequestAttributesMissing RequestAttributesMissing;
} StartDisclosureResultKind;

typedef struct wire_cst_start_disclosure_result {
  int32_t tag;
  union StartDisclosureResultKind kind;
} wire_cst_start_disclosure_result;

typedef struct wire_cst_WalletInstructionResult_InstructionError {
  struct wire_cst_wallet_instruction_error *error;
} wire_cst_WalletInstructionResult_InstructionError;

typedef union WalletInstructionResultKind {
  struct wire_cst_WalletInstructionResult_InstructionError InstructionError;
} WalletInstructionResultKind;

typedef struct wire_cst_wallet_instruction_result {
  int32_t tag;
  union WalletInstructionResultKind kind;
} wire_cst_wallet_instruction_result;

void frbgen_wallet_core_wire__crate__api__full__accept_disclosure(int64_t port_,
                                                                  struct wire_cst_list_prim_u_8_strict *pin);

void frbgen_wallet_core_wire__crate__api__full__accept_issuance(int64_t port_,
                                                                struct wire_cst_list_prim_u_8_strict *pin);

void frbgen_wallet_core_wire__crate__api__full__cancel_disclosure(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__cancel_issuance(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__change_pin(int64_t port_,
                                                           struct wire_cst_list_prim_u_8_strict *old_pin,
                                                           struct wire_cst_list_prim_u_8_strict *new_pin);

void frbgen_wallet_core_wire__crate__api__full__check_pin(int64_t port_,
                                                          struct wire_cst_list_prim_u_8_strict *pin);

void frbgen_wallet_core_wire__crate__api__full__clear_attestations_stream(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__clear_configuration_stream(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__clear_lock_stream(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__clear_recent_history_stream(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__clear_version_state_stream(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__continue_change_pin(int64_t port_,
                                                                    struct wire_cst_list_prim_u_8_strict *pin);

void frbgen_wallet_core_wire__crate__api__full__continue_disclosure_based_issuance(int64_t port_,
                                                                                   struct wire_cst_list_prim_u_8_strict *pin);

void frbgen_wallet_core_wire__crate__api__full__continue_pid_issuance(int64_t port_,
                                                                      struct wire_cst_list_prim_u_8_strict *uri);

void frbgen_wallet_core_wire__crate__api__full__create_pid_issuance_redirect_uri(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__get_history(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__get_history_for_card(int64_t port_,
                                                                     struct wire_cst_list_prim_u_8_strict *attestation_type);

void frbgen_wallet_core_wire__crate__api__full__get_version_string(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__has_active_disclosure_session(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__has_active_issuance_session(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__has_registration(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__identify_uri(int64_t port_,
                                                             struct wire_cst_list_prim_u_8_strict *uri);

void frbgen_wallet_core_wire__crate__api__full__init(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__is_biometric_unlock_enabled(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__is_initialized(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__is_valid_pin(int64_t port_,
                                                             struct wire_cst_list_prim_u_8_strict *pin);

void frbgen_wallet_core_wire__crate__api__full__lock_wallet(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__register(int64_t port_,
                                                         struct wire_cst_list_prim_u_8_strict *pin);

void frbgen_wallet_core_wire__crate__api__full__reset_wallet(int64_t port_);

void frbgen_wallet_core_wire__crate__api__full__set_attestations_stream(int64_t port_,
                                                                        struct wire_cst_list_prim_u_8_strict *sink);

void frbgen_wallet_core_wire__crate__api__full__set_biometric_unlock(int64_t port_, bool enable);

void frbgen_wallet_core_wire__crate__api__full__set_configuration_stream(int64_t port_,
                                                                         struct wire_cst_list_prim_u_8_strict *sink);

void frbgen_wallet_core_wire__crate__api__full__set_lock_stream(int64_t port_,
                                                                struct wire_cst_list_prim_u_8_strict *sink);

void frbgen_wallet_core_wire__crate__api__full__set_recent_history_stream(int64_t port_,
                                                                          struct wire_cst_list_prim_u_8_strict *sink);

void frbgen_wallet_core_wire__crate__api__full__set_version_state_stream(int64_t port_,
                                                                         struct wire_cst_list_prim_u_8_strict *sink);

void frbgen_wallet_core_wire__crate__api__full__start_disclosure(int64_t port_,
                                                                 struct wire_cst_list_prim_u_8_strict *uri,
                                                                 bool is_qr_code);

void frbgen_wallet_core_wire__crate__api__full__unlock_wallet(int64_t port_,
                                                              struct wire_cst_list_prim_u_8_strict *pin);

void frbgen_wallet_core_wire__crate__api__full__unlock_wallet_with_biometrics(int64_t port_);

struct wire_cst_attestation *frbgen_wallet_core_cst_new_box_autoadd_attestation(void);

struct wire_cst_image *frbgen_wallet_core_cst_new_box_autoadd_image(void);

struct wire_cst_image_with_metadata *frbgen_wallet_core_cst_new_box_autoadd_image_with_metadata(void);

struct wire_cst_organization *frbgen_wallet_core_cst_new_box_autoadd_organization(void);

struct wire_cst_rendering_metadata *frbgen_wallet_core_cst_new_box_autoadd_rendering_metadata(void);

struct wire_cst_request_policy *frbgen_wallet_core_cst_new_box_autoadd_request_policy(void);

uint64_t *frbgen_wallet_core_cst_new_box_autoadd_u_64(uint64_t value);

struct wire_cst_wallet_instruction_error *frbgen_wallet_core_cst_new_box_autoadd_wallet_instruction_error(void);

struct wire_cst_list_attestation *frbgen_wallet_core_cst_new_list_attestation(int32_t len);

struct wire_cst_list_attestation_attribute *frbgen_wallet_core_cst_new_list_attestation_attribute(int32_t len);

struct wire_cst_list_claim_display_metadata *frbgen_wallet_core_cst_new_list_claim_display_metadata(int32_t len);

struct wire_cst_list_display_metadata *frbgen_wallet_core_cst_new_list_display_metadata(int32_t len);

struct wire_cst_list_localized_string *frbgen_wallet_core_cst_new_list_localized_string(int32_t len);

struct wire_cst_list_missing_attribute *frbgen_wallet_core_cst_new_list_missing_attribute(int32_t len);

struct wire_cst_list_prim_u_8_strict *frbgen_wallet_core_cst_new_list_prim_u_8_strict(int32_t len);

struct wire_cst_list_wallet_event *frbgen_wallet_core_cst_new_list_wallet_event(int32_t len);
static int64_t dummy_method_to_enforce_bundling(void) {
    int64_t dummy_var = 0;
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_box_autoadd_attestation);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_box_autoadd_image);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_box_autoadd_image_with_metadata);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_box_autoadd_organization);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_box_autoadd_rendering_metadata);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_box_autoadd_request_policy);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_box_autoadd_u_64);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_box_autoadd_wallet_instruction_error);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_list_attestation);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_list_attestation_attribute);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_list_claim_display_metadata);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_list_display_metadata);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_list_localized_string);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_list_missing_attribute);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_list_prim_u_8_strict);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_cst_new_list_wallet_event);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__accept_disclosure);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__accept_issuance);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__cancel_disclosure);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__cancel_issuance);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__change_pin);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__check_pin);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__clear_attestations_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__clear_configuration_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__clear_lock_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__clear_recent_history_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__clear_version_state_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__continue_change_pin);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__continue_disclosure_based_issuance);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__continue_pid_issuance);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__create_pid_issuance_redirect_uri);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__get_history);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__get_history_for_card);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__get_version_string);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__has_active_disclosure_session);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__has_active_issuance_session);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__has_registration);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__identify_uri);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__init);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__is_biometric_unlock_enabled);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__is_initialized);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__is_valid_pin);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__lock_wallet);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__register);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__reset_wallet);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__set_attestations_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__set_biometric_unlock);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__set_configuration_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__set_lock_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__set_recent_history_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__set_version_state_stream);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__start_disclosure);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__unlock_wallet);
    dummy_var ^= ((int64_t) (void*) frbgen_wallet_core_wire__crate__api__full__unlock_wallet_with_biometrics);
    dummy_var ^= ((int64_t) (void*) store_dart_post_cobject);
    return dummy_var;
}
