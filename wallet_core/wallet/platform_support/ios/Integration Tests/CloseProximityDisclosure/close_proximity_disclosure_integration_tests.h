//
//  close_proximity_disclosure_integration_tests.h
//  CloseProximityDisclosure
//
//  Created by The Wallet Developers on 06/03/2026.
//

#ifndef close_proximity_disclosure_integration_tests_h
#define close_proximity_disclosure_integration_tests_h

void close_proximity_disclosure_test_start_qr_handover();

// Returns NULL on success, or an owned error string. The callback string is borrowed.
char *wallet_close_proximity_disclosure_test(void (*reader_ready)(const char *));
char *wallet_jwt_registration_certificate_test(void (*reader_ready)(const char *));
char *wallet_registration_certificate_test(const char *test_case, void (*reader_ready)(const char *));
void wallet_device_test_free_error(char *error);

#endif /* close_proximity_disclosure_integration_tests_h */
