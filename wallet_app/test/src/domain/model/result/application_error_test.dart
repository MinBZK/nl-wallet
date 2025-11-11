import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/pin/check_pin_result.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

void main() {
  group('ApplicationError', () {
    final sourceError = Exception('source');
    final sourceError2 = Exception('source2');

    group('GenericError', () {
      test('props match', () {
        expect(
          GenericError('message', redirectUrl: 'url', sourceError: sourceError),
          GenericError('message', redirectUrl: 'url', sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          GenericError('message', redirectUrl: 'url', sourceError: sourceError),
          isNot(GenericError('message2', redirectUrl: 'url', sourceError: sourceError)),
        );
        expect(
          GenericError('message', redirectUrl: 'url', sourceError: sourceError),
          isNot(GenericError('message', redirectUrl: 'url2', sourceError: sourceError)),
        );
        expect(
          GenericError('message', redirectUrl: 'url', sourceError: sourceError),
          isNot(GenericError('message', redirectUrl: 'url', sourceError: sourceError2)),
        );
      });
    });

    group('NetworkError', () {
      test('props match', () {
        expect(
          NetworkError(hasInternet: true, statusCode: 200, sourceError: sourceError),
          NetworkError(hasInternet: true, statusCode: 200, sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          NetworkError(hasInternet: true, statusCode: 200, sourceError: sourceError),
          isNot(NetworkError(hasInternet: false, statusCode: 200, sourceError: sourceError)),
        );
        expect(
          NetworkError(hasInternet: true, statusCode: 200, sourceError: sourceError),
          isNot(NetworkError(hasInternet: true, statusCode: 404, sourceError: sourceError)),
        );
        expect(
          NetworkError(hasInternet: true, statusCode: 200, sourceError: sourceError),
          isNot(NetworkError(hasInternet: true, statusCode: 200, sourceError: sourceError2)),
        );
      });
    });

    group('ValidatePinError', () {
      test('props match', () {
        expect(
          ValidatePinError(PinValidationError.tooFewUniqueDigits, sourceError: sourceError),
          ValidatePinError(PinValidationError.tooFewUniqueDigits, sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          ValidatePinError(PinValidationError.tooFewUniqueDigits, sourceError: sourceError),
          isNot(ValidatePinError(PinValidationError.sequentialDigits, sourceError: sourceError)),
        );
        expect(
          ValidatePinError(PinValidationError.tooFewUniqueDigits, sourceError: sourceError),
          isNot(ValidatePinError(PinValidationError.tooFewUniqueDigits, sourceError: sourceError2)),
        );
      });
    });

    group('CheckPinError', () {
      test('props match', () {
        expect(
          CheckPinError(CheckPinResultIncorrect(attemptsLeftInRound: 3), sourceError: sourceError),
          CheckPinError(CheckPinResultIncorrect(attemptsLeftInRound: 3), sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          CheckPinError(CheckPinResultIncorrect(attemptsLeftInRound: 3), sourceError: sourceError),
          isNot(CheckPinError(CheckPinResultIncorrect(attemptsLeftInRound: 2), sourceError: sourceError)),
        );
        expect(
          CheckPinError(CheckPinResultIncorrect(attemptsLeftInRound: 3), sourceError: sourceError),
          isNot(CheckPinError(CheckPinResultIncorrect(attemptsLeftInRound: 3), sourceError: sourceError2)),
        );
      });
    });

    group('HardwareUnsupportedError', () {
      test('props match', () {
        expect(
          HardwareUnsupportedError(sourceError: sourceError),
          HardwareUnsupportedError(sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          HardwareUnsupportedError(sourceError: sourceError),
          isNot(HardwareUnsupportedError(sourceError: sourceError2)),
        );
      });
    });

    group('SessionError', () {
      test('props match', () {
        expect(
          SessionError(state: SessionState.expired, sourceError: sourceError),
          SessionError(state: SessionState.expired, sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          SessionError(state: SessionState.expired, sourceError: sourceError),
          isNot(SessionError(state: SessionState.cancelled, sourceError: sourceError)),
        );
        expect(
          SessionError(state: SessionState.expired, sourceError: sourceError),
          isNot(SessionError(state: SessionState.expired, sourceError: sourceError2)),
        );
      });
    });

    group('RedirectUriError', () {
      test('props match', () {
        expect(
          RedirectUriError(redirectError: RedirectError.loginRequired, sourceError: sourceError),
          RedirectUriError(redirectError: RedirectError.loginRequired, sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          RedirectUriError(redirectError: RedirectError.loginRequired, sourceError: sourceError),
          isNot(RedirectUriError(redirectError: RedirectError.accessDenied, sourceError: sourceError)),
        );
        expect(
          RedirectUriError(redirectError: RedirectError.loginRequired, sourceError: sourceError),
          isNot(RedirectUriError(redirectError: RedirectError.loginRequired, sourceError: sourceError2)),
        );
      });
    });

    group('ExternalScannerError', () {
      test('props match', () {
        expect(
          ExternalScannerError(sourceError: sourceError),
          ExternalScannerError(sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          ExternalScannerError(sourceError: sourceError),
          isNot(ExternalScannerError(sourceError: sourceError2)),
        );
      });
    });

    group('WrongDigidError', () {
      test('props match', () {
        expect(
          WrongDigidError(sourceError: sourceError),
          WrongDigidError(sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          WrongDigidError(sourceError: sourceError),
          isNot(WrongDigidError(sourceError: sourceError2)),
        );
      });
    });

    group('DeniedDigidError', () {
      test('props match', () {
        expect(
          DeniedDigidError(sourceError: sourceError),
          DeniedDigidError(sourceError: sourceError),
        );
      });

      test('props do not match', () {
        expect(
          DeniedDigidError(sourceError: sourceError),
          isNot(DeniedDigidError(sourceError: sourceError2)),
        );
      });
    });

    group('RelyingPartyError', () {
      test('props match', () {
        expect(
          RelyingPartyError(sourceError: sourceError, organizationName: {const Locale('en'): 'org'}),
          RelyingPartyError(sourceError: sourceError, organizationName: {const Locale('en'): 'org'}),
        );
      });

      test('props do not match', () {
        expect(
          RelyingPartyError(sourceError: sourceError, organizationName: {const Locale('en'): 'org'}),
          isNot(
            RelyingPartyError(sourceError: sourceError, organizationName: {const Locale('en'): 'org2'}),
          ),
        );
        expect(
          RelyingPartyError(sourceError: sourceError, organizationName: {const Locale('en'): 'org'}),
          isNot(
            RelyingPartyError(sourceError: sourceError2, organizationName: {const Locale('en'): 'org'}),
          ),
        );
      });
    });
  });
}
