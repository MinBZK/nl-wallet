import 'dart:convert';

import '../../util/mapper/mapper.dart';
import 'core_error.dart';
import 'flutter_api_error.dart';

/// Maps a 'FlutterApiErrorJson' [String] to a [CoreError].
class CoreErrorMapper extends Mapper<String, CoreError> {
  @override
  CoreError map(String input) {
    final decodedJson = json.decode(input);
    final flutterApiError = FlutterApiError.fromJson(decodedJson);
    switch (flutterApiError.type) {
      case FlutterApiErrorType.generic:
        return CoreGenericError(flutterApiError.description);
      case FlutterApiErrorType.networking:
        return CoreNetworkError(flutterApiError.description);
      case FlutterApiErrorType.walletState:
        return CoreStateError(flutterApiError.description, flutterApiError.data);
      case FlutterApiErrorType.redirectUri:
        return CoreRedirectUriError(
          flutterApiError.description,
          redirectError: _mapRedirectError(flutterApiError.data),
        );
      case FlutterApiErrorType.hardwareKeyUnsupported:
        return CoreHardwareKeyUnsupportedError(flutterApiError.description);
      case FlutterApiErrorType.disclosureSourceMismatch:
        final isCrossDevice = flutterApiError.data?['session_type'] == 'cross_device';
        return CoreDisclosureSourceMismatchError(
          flutterApiError.description,
          isCrossDevice: isCrossDevice,
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
