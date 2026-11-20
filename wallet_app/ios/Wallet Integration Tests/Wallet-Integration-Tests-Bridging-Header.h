#include "../../../wallet_core/wallet/platform_support/ios/PlatformSupport/PlatformSupport-Bridging-Header.h"

// Returns NULL on success, or an owned error string. The callback string is borrowed.
char *wallet_close_proximity_disclosure_test(void (*reader_ready)(const char *));
char *wallet_jwt_registration_certificate_test(void (*reader_ready)(const char *));
char *wallet_registration_certificate_test(const char *test_case, void (*reader_ready)(const char *));
void wallet_device_test_free_error(char *error);
