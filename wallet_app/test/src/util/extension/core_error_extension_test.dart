import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/util/extension/core_error_extension.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';
import 'package:wallet_core/core.dart';

import '../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockNetworkRepository mockNetworkRepository;

  setUp(() {
    mockNetworkRepository = MockNetworkRepository();
    CoreErrorExtension.networkRepository = mockNetworkRepository;
  });

  group('CoreErrorExtension', () {
    test('CoreGenericError maps to GenericError', () async {
      const coreError = CoreGenericError('description', data: {'return_url': 'url'});
      final applicationError = await coreError.asApplicationError();

      expect(applicationError, isA<GenericError>());
      final genericError = applicationError as GenericError;
      expect(genericError.rawMessage, 'description');
      expect(genericError.redirectUrl, 'url');
      expect(genericError.sourceError, coreError);
    });

    group('CoreNetworkError', () {
      test('maps to NetworkError with internet', () async {
        when(mockNetworkRepository.hasInternet()).thenAnswer((_) async => true);
        const coreError = CoreNetworkError('description');

        final applicationError = await coreError.asApplicationError();

        expect(applicationError, isA<NetworkError>());
        final networkError = applicationError as NetworkError;
        expect(networkError.hasInternet, isTrue);
        expect(networkError.sourceError, coreError);
      });

      test('maps to NetworkError without internet', () async {
        when(mockNetworkRepository.hasInternet()).thenAnswer((_) async => false);
        const coreError = CoreNetworkError('description');

        final applicationError = await coreError.asApplicationError();

        expect(applicationError, isA<NetworkError>());
        final networkError = applicationError as NetworkError;
        expect(networkError.hasInternet, isFalse);
        expect(networkError.sourceError, coreError);
      });
    });

    test('CoreRedirectUriError maps to RedirectUriError', () async {
      const coreError = CoreRedirectUriError('description', redirectError: RedirectError.accessDenied);

      final applicationError = await coreError.asApplicationError();

      expect(applicationError, isA<RedirectUriError>());
      final redirectError = applicationError as RedirectUriError;
      expect(redirectError.redirectError, RedirectError.accessDenied);
      expect(redirectError.sourceError, coreError);
    });

    test('CoreHardwareKeyUnsupportedError maps to HardwareUnsupportedError', () async {
      const coreError = CoreHardwareKeyUnsupportedError('description');

      final applicationError = await coreError.asApplicationError();

      expect(applicationError, isA<HardwareUnsupportedError>());
      expect(applicationError.sourceError, coreError);
    });

    group('CoreDisclosureSourceMismatchError', () {
      test('maps to ExternalScannerError when isCrossDevice is true', () async {
        const coreError = CoreDisclosureSourceMismatchError('description', isCrossDevice: true);

        final applicationError = await coreError.asApplicationError();

        expect(applicationError, isA<ExternalScannerError>());
        expect(applicationError.sourceError, coreError);
      });

      test('maps to GenericError when isCrossDevice is false', () async {
        const coreError = CoreDisclosureSourceMismatchError(
          'description',
          isCrossDevice: false,
          data: {'return_url': 'url'},
        );

        final applicationError = await coreError.asApplicationError();

        expect(applicationError, isA<GenericError>());
        final genericError = applicationError as GenericError;
        expect(genericError.rawMessage, 'description');
        expect(genericError.redirectUrl, 'url');
        expect(genericError.sourceError, coreError);
      });
    });

    test('CoreExpiredSessionError maps to SessionError', () async {
      const coreError = CoreExpiredSessionError('description', canRetry: true, data: {'return_url': 'url'});

      final applicationError = await coreError.asApplicationError();

      expect(applicationError, isA<SessionError>());
      final sessionError = applicationError as SessionError;
      expect(sessionError.state, SessionState.expired);
      expect(sessionError.canRetry, isTrue);
      expect(sessionError.returnUrl, 'url');
      expect(sessionError.sourceError, coreError);
    });

    test('CoreCancelledSessionError maps to SessionError', () async {
      const coreError = CoreCancelledSessionError('description', data: {'return_url': 'url'});

      final applicationError = await coreError.asApplicationError();

      expect(applicationError, isA<SessionError>());
      final sessionError = applicationError as SessionError;
      expect(sessionError.state, SessionState.cancelled);
      expect(sessionError.canRetry, isFalse);
      expect(sessionError.returnUrl, 'url');
      expect(sessionError.sourceError, coreError);
    });

    group('CoreRelyingPartyError', () {
      test('maps to RelyingPartyError with organization name', () async {
        final coreError = CoreRelyingPartyError(
          'description',
          organizationName: [const LocalizedString(language: 'en', value: 'org')],
        );

        final applicationError = await coreError.asApplicationError();

        expect(applicationError, isA<RelyingPartyError>());
        final rpError = applicationError as RelyingPartyError;
        expect(rpError.organizationName, {const Locale('en'): 'org'});
        expect(rpError.sourceError, coreError);
      });

      test('maps to RelyingPartyError without organization name', () async {
        final coreError = CoreRelyingPartyError('description');

        final applicationError = await coreError.asApplicationError();

        expect(applicationError, isA<RelyingPartyError>());
        final rpError = applicationError as RelyingPartyError;
        expect(rpError.organizationName, isNull);
        expect(rpError.sourceError, coreError);
      });
    });

    test('CoreWrongDigidError maps to WrongDigidError', () async {
      const coreError = CoreWrongDigidError('description');

      final applicationError = await coreError.asApplicationError();

      expect(applicationError, isA<WrongDigidError>());
      expect(applicationError.sourceError, coreError);
    });

    test('CoreDeniedDigidError maps to DeniedDigidError', () async {
      const coreError = CoreDeniedDigidError('description');

      final applicationError = await coreError.asApplicationError();

      expect(applicationError, isA<DeniedDigidError>());
      expect(applicationError.sourceError, coreError);
    });

    test('CoreStateError throws StateError', () async {
      const coreError = CoreStateError('description');

      expect(() async => coreError.asApplicationError(), throwsA(isA<StateError>()));
    });
  });
}
