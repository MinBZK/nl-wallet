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
    return _mapErrorType(flutterApiError);
  }

  CoreError _mapErrorType(FlutterApiError error) {
    return switch (error.type) {
      FlutterApiErrorType.generic ||
      FlutterApiErrorType.server => CoreGenericError(error.description, data: error.data),
      FlutterApiErrorType.networking => CoreNetworkError(error.description, data: error.data),
      FlutterApiErrorType.walletState => CoreStateError(error.description, data: error.data),
      FlutterApiErrorType.redirectUri => _mapRedirectUriError(error),
      FlutterApiErrorType.hardwareKeyUnsupported => CoreHardwareKeyUnsupportedError(
        error.description,
        data: error.data,
      ),
      FlutterApiErrorType.disclosureSourceMismatch => _mapDisclosureSourceMismatchError(error),
      FlutterApiErrorType.expiredSession => _mapExpiredSessionError(error),
      FlutterApiErrorType.cancelledSession => CoreCancelledSessionError(error.description, data: error.data),
      FlutterApiErrorType.issuer || FlutterApiErrorType.verifier => _mapRelyingPartyError(error),
      FlutterApiErrorType.wrongDigid => CoreWrongDigidError(error.description),
      FlutterApiErrorType.deniedDigid => CoreDeniedDigidError(error.description),
    };
  }

  CoreError _mapRedirectUriError(FlutterApiError error) {
    return CoreRedirectUriError(
      error.description,
      data: error.data,
      redirectError: _extractRedirectError(error.data),
    );
  }

  CoreError _mapDisclosureSourceMismatchError(FlutterApiError error) {
    final isCrossDevice = error.data?['session_type'] == 'cross_device';
    return CoreDisclosureSourceMismatchError(
      error.description,
      data: error.data,
      isCrossDevice: isCrossDevice,
    );
  }

  CoreError _mapExpiredSessionError(FlutterApiError error) {
    final canRetry = error.data?['can_retry'] == true;
    return CoreExpiredSessionError(
      error.description,
      canRetry: canRetry,
      data: error.data,
    );
  }

  CoreError _mapRelyingPartyError(FlutterApiError error) {
    final organizationName = tryCast<Map<String, dynamic>>(error.data?['organization_name']);
    final localizedStrings = _parseLocalizedStrings(organizationName);

    return CoreRelyingPartyError(
      error.description,
      data: error.data,
      organizationName: localizedStrings.takeIf((it) => it.isNotEmpty),
    );
  }

  RedirectError _extractRedirectError(Map<String, dynamic>? data) {
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

  List<LocalizedString> _parseLocalizedStrings(Map<String, dynamic>? organizationName) {
    final List<LocalizedString> localizedStrings = [];
    organizationName?.forEach(
      (key, value) => localizedStrings.add(LocalizedString(language: key, value: value)),
    );
    return localizedStrings;
  }
}
