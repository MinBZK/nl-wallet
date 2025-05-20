import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';
import 'package:wallet/src/wallet_core/error/core_error_mapper.dart';
import 'package:wallet/src/wallet_core/error/flutter_api_error.dart';

void main() {
  const defaultDescription = 'core error description';
  late CoreErrorMapper errorMapper;

  setUp(() {
    errorMapper = CoreErrorMapper();
  });

  group('map', () {
    test('mapping empty string throws', () {
      expect(() => errorMapper.map(''), throwsException);
    });

    test('mapping invalid string throws', () {
      expect(() => errorMapper.map('this is not valid json'), throwsException);
    });

    test('mapping FlutterApiErrorType.generic results in CoreGenericError', () {
      final error = FlutterApiError(type: FlutterApiErrorType.generic, description: defaultDescription, data: null);
      final errorJson = jsonEncode(error);
      final result = errorMapper.map(errorJson);
      expect(result, const CoreGenericError(defaultDescription));
    });

    test('mapping FlutterApiErrorType.server results in CoreGenericError', () {
      // Server errors don't require any dedicated handling just yet (rust side is still defining when exactly this is relevant). So
      // it should be treated as a generic error for now.
      final error = FlutterApiError(
        type: FlutterApiErrorType.server,
        description: defaultDescription,
        data: {'http_error': 'some extra error information might show up here'},
      );
      final errorJson = jsonEncode(error);
      final result = errorMapper.map(errorJson);
      expect(result, CoreGenericError(defaultDescription, data: error.data));
    });

    test('mapping FlutterApiErrorType.walletState results in CoreStateError', () {
      final error = FlutterApiError(type: FlutterApiErrorType.walletState, description: defaultDescription, data: null);
      final errorJson = jsonEncode(error);
      final result = errorMapper.map(errorJson);
      expect(result, const CoreStateError(defaultDescription));
    });

    test('mapping FlutterApiErrorType.networking results in CoreNetworkError', () {
      final error = FlutterApiError(type: FlutterApiErrorType.networking, description: defaultDescription, data: null);
      final errorJson = jsonEncode(error);
      final result = errorMapper.map(errorJson);
      expect(result, const CoreNetworkError(defaultDescription));
    });

    test('mapping FlutterApiErrorType.redirectUri results in CoreRedirectUriError with RedirectError.unknown', () {
      final error = FlutterApiError(type: FlutterApiErrorType.redirectUri, description: defaultDescription, data: null);
      final errorJson = jsonEncode(error);
      final result = errorMapper.map(errorJson);
      expect(result, const CoreRedirectUriError(defaultDescription, redirectError: RedirectError.unknown));
    });

    test(
        'mapping FlutterApiErrorType.redirectUri with invalid data results in CoreRedirectUriError with RedirectError.unknown',
        () {
      final error = FlutterApiError(
        type: FlutterApiErrorType.redirectUri,
        description: defaultDescription,
        data: {'redirect_error': 'xxyyzz'},
      );
      final errorJson = jsonEncode(error);
      final result = errorMapper.map(errorJson);
      expect(result, CoreRedirectUriError(defaultDescription, redirectError: RedirectError.unknown, data: error.data));
    });

    test(
        'mapping FlutterApiErrorType.redirectUri with accessDenied data results in CoreRedirectUriError with RedirectError.accessDenied',
        () {
      final error = FlutterApiError(
        type: FlutterApiErrorType.redirectUri,
        description: defaultDescription,
        data: {'redirect_error': 'access_denied'},
      );
      final errorJson = jsonEncode(error);
      final result = errorMapper.map(errorJson);
      expect(
        result,
        CoreRedirectUriError(defaultDescription, redirectError: RedirectError.accessDenied, data: error.data),
      );
    });

    test(
        'mapping FlutterApiErrorType.redirectUri with serverError data results in CoreRedirectUriError with RedirectError.serverError',
        () {
      final error = FlutterApiError(
        type: FlutterApiErrorType.redirectUri,
        description: defaultDescription,
        data: {'redirect_error': 'server_error'},
      );
      final errorJson = jsonEncode(error);
      final result = errorMapper.map(errorJson);
      expect(
        result,
        CoreRedirectUriError(defaultDescription, redirectError: RedirectError.serverError, data: error.data),
      );
    });

    test(
        'mapping a FlutterApiErrorType.expiredSession with canRetry data set to true results in CoreExpiredSessionError with canRetry=true',
        () {
      final Map<String, dynamic> data = {'can_retry': true};
      final error = FlutterApiError(
        type: FlutterApiErrorType.expiredSession,
        description: defaultDescription,
        data: data,
      );
      final errorJson = jsonEncode(error);
      final result = errorMapper.map(errorJson);
      expect(result, CoreExpiredSessionError(defaultDescription, canRetry: true, data: data));
    });
  });
}
