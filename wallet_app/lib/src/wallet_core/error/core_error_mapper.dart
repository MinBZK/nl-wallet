import 'dart:convert';

import 'core_error.dart';
import 'flutter_api_error.dart';

class CoreErrorMapper {
  CoreError map(String flutterApiErrorJson) {
    final decodedJson = json.decode(flutterApiErrorJson);
    final flutterApiError = FlutterApiError.fromJson(decodedJson);
    switch (flutterApiError.type) {
      case FlutterApiErrorType.generic:
        return CoreGenericError(flutterApiError.description);
      case FlutterApiErrorType.networking:
        return CoreNetworkError(flutterApiError.description);
      case FlutterApiErrorType.redirectUri:
        return CoreRedirectUriError(
          flutterApiError.description,
          redirectError: _mapRedirectError(flutterApiError.data),
        );
    }
  }

  RedirectError _mapRedirectError(Map<String, dynamic>? data) {
    switch (data?['redirect_error']) {
      case 'access_denied':
        return RedirectError.accessDenied;
      case 'server_error':
        return RedirectError.serverError;
      default:
        return RedirectError.unknown;
    }
  }
}
