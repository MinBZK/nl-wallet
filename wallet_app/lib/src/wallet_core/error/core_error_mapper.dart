import 'dart:convert';

import 'package:wallet_core/core.dart';

import '../../util/cast_util.dart';
import '../../util/extension/object_extension.dart';
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
      case FlutterApiErrorType.server:
        return CoreGenericError(flutterApiError.description, data: flutterApiError.data);
      case FlutterApiErrorType.networking:
        return CoreNetworkError(flutterApiError.description, data: flutterApiError.data);
      case FlutterApiErrorType.walletState:
        return CoreStateError(flutterApiError.description, data: flutterApiError.data);
      case FlutterApiErrorType.redirectUri:
        return CoreRedirectUriError(
          flutterApiError.description,
          data: flutterApiError.data,
          redirectError: _mapRedirectError(flutterApiError.data),
        );
      case FlutterApiErrorType.hardwareKeyUnsupported:
        return CoreHardwareKeyUnsupportedError(flutterApiError.description, data: flutterApiError.data);
      case FlutterApiErrorType.disclosureSourceMismatch:
        final isCrossDevice = flutterApiError.data?['session_type'] == 'cross_device';
        return CoreDisclosureSourceMismatchError(
          flutterApiError.description,
          data: flutterApiError.data,
          isCrossDevice: isCrossDevice,
        );
      case FlutterApiErrorType.expiredSession:
        final canRetry = flutterApiError.data?['can_retry'] == true;
        return CoreExpiredSessionError(
          flutterApiError.description,
          canRetry: canRetry,
          data: flutterApiError.data,
        );
      case FlutterApiErrorType.cancelledSession:
        return CoreCancelledSessionError(flutterApiError.description, data: flutterApiError.data);
      case FlutterApiErrorType.issuer:
      case FlutterApiErrorType.verifier:
        final organizationName = tryCast<Map<String, dynamic>>(flutterApiError.data?['organization_name']);
        final List<LocalizedString> localizedStrings = [];
        organizationName?.forEach((key, value) => localizedStrings.add(LocalizedString(language: key, value: value)));
        return CoreRelyingPartyError(
          flutterApiError.description,
          data: flutterApiError.data,
          organizationName: localizedStrings.takeIf((it) => it.isNotEmpty),
        );
    }
  }

  RedirectError _mapRedirectError(Map<String, dynamic>? data) {
    switch (data?['redirect_error']) {
      case 'access_denied':
        return RedirectError.accessDenied;
      case 'server_error':
        return RedirectError.serverError;
      case 'login_required':
        return RedirectError.loginRequired;
      default:
        return RedirectError.unknown;
    }
  }
}
